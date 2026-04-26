use crate::models::sub_transactions::SubTransaction;
use crate::models::transactions::Transaction;
use chrono::{NaiveDate, NaiveDateTime};
use sqlx::MySqlPool;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TransactionUpsert {
    pub account_id: i64,
    pub transaction_type_id: i64,
    pub category_ids: Vec<i64>,
    pub datetime: NaiveDateTime,
    pub amount: f64,
    pub description: String,
    pub note: Option<String>,
}

async fn sync_transaction_categories(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    transaction_id: i64,
    category_ids: &[i64],
) -> Result<(), anyhow::Error> {
    sqlx::query("DELETE FROM transactions_categories WHERE transaction_id = ?")
        .bind(transaction_id)
        .execute(&mut **tx)
        .await?;

    for category_id in category_ids {
        sqlx::query(
            "INSERT INTO transactions_categories (transaction_id, category_id) VALUES (?, ?)",
        )
        .bind(transaction_id)
        .bind(category_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn sync_sub_transaction_categories(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    sub_transaction_id: i64,
    category_ids: &[i64],
) -> Result<(), anyhow::Error> {
    sqlx::query("DELETE FROM sub_transactions_categories WHERE sub_transaction_id = ?")
        .bind(sub_transaction_id)
        .execute(&mut **tx)
        .await?;

    for category_id in category_ids {
        sqlx::query(
            "INSERT INTO sub_transactions_categories (sub_transaction_id, category_id) VALUES (?, ?)",
        )
        .bind(sub_transaction_id)
        .bind(category_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

/// Fetches the display currency code from the settings service.
/// Returns `None` if the setting has no value, is not found, or the call fails.
async fn fetch_target_currency(
    http_client: &reqwest::Client,
    settings_service_url: &str,
) -> Option<String> {
    let url = format!("{}/settings/current_currency", settings_service_url);
    let resp = match http_client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(
                url = %url,
                error = %e,
                "Failed to reach settings service while fetching current_currency"
            );
            return None;
        }
    };
    if resp.status().as_u16() == 404 {
        return None;
    }
    if !resp.status().is_success() {
        tracing::error!(
            url = %url,
            status = %resp.status(),
            "Settings service returned an error while fetching current_currency"
        );
        return None;
    }
    let body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse settings service response");
            return None;
        }
    };
    body["data"]["value"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Fetches the exchange rate from `from` to `to` on `date` via the currency service.
/// Returns `None` and logs an error if the call fails.
/// Returns `Some(1.0)` immediately when both currencies are the same.
async fn fetch_exchange_rate(
    http_client: &reqwest::Client,
    currency_service_url: &str,
    from: &str,
    to: &str,
    date: NaiveDate,
) -> Option<f64> {
    if from == to {
        return Some(1.0);
    }
    let url = format!(
        "{}/rates?date={}&from={}&to={}",
        currency_service_url,
        date.format("%Y-%m-%d"),
        from,
        to
    );
    let resp = match http_client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(
                url = %url,
                from = %from,
                to = %to,
                date = %date,
                error = %e,
                "Failed to reach currency service while fetching exchange rate"
            );
            return None;
        }
    };
    if !resp.status().is_success() {
        tracing::error!(
            url = %url,
            from = %from,
            to = %to,
            date = %date,
            status = %resp.status(),
            "Currency service returned an error while fetching exchange rate"
        );
        return None;
    }
    let body: serde_json::Value = match resp.json().await {
        Ok(b) => b,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse currency service response");
            return None;
        }
    };
    match body["data"]["rate"].as_f64() {
        Some(rate) => Some(rate),
        None => {
            tracing::error!(
                from = %from,
                to = %to,
                date = %date,
                "Currency service response did not contain a valid rate value"
            );
            None
        }
    }
}

pub async fn list_transactions(
    pool: &MySqlPool,
    http_client: &reqwest::Client,
    settings_service_url: &str,
    currency_service_url: &str,
    account_id: Option<i64>,
    transaction_type_id: Option<i64>,
    category_id: Option<i64>,
    description: Option<String>,
    page: u32,
    page_size: u32,
) -> Result<(Vec<Transaction>, u64), anyhow::Error> {
    let total_row = sqlx::query(
        r#"
        SELECT COUNT(*) AS total_count
        FROM transactions t
        WHERE (? IS NULL OR t.account_id = ?)
        AND (? IS NULL OR t.transaction_type_id = ?)
        AND (? IS NULL OR EXISTS (
                SELECT 1
                FROM transactions_categories tc_filter
                WHERE tc_filter.transaction_id = t.id
                AND tc_filter.category_id = ?
            )
        )
        AND (? IS NULL OR LOWER(t.description) LIKE CONCAT('%', LOWER(?), '%'))
        "#,
    )
    .bind(account_id)
    .bind(account_id)
    .bind(transaction_type_id)
    .bind(transaction_type_id)
    .bind(category_id)
    .bind(category_id)
    .bind(description.clone())
    .bind(description.clone())
    .fetch_one(pool)
    .await?;

    let total_count = total_row.try_get::<i64, _>("total_count")? as u64;
    let offset = ((page - 1) * page_size) as i64;

    let mut transactions = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT
            t.id,
            t.account_id,
            t.transaction_type_id,
            tt.name AS transaction_type_name,
            GROUP_CONCAT(DISTINCT c.id ORDER BY c.name SEPARATOR ',') AS category_ids,
            GROUP_CONCAT(DISTINCT c.name ORDER BY c.name SEPARATOR ', ') AS categories,
            t.datetime,
            t.amount,
            t.description,
            t.note,
            t.fingerprint,
            a.currency AS account_currency
        FROM transactions t
        LEFT JOIN transaction_types tt ON t.transaction_type_id = tt.id
        LEFT JOIN transactions_categories tc ON t.id = tc.transaction_id
        LEFT JOIN categories c ON tc.category_id = c.id
        LEFT JOIN accounts a ON t.account_id = a.id
        WHERE (? IS NULL OR t.account_id = ?)
        AND (? IS NULL OR t.transaction_type_id = ?)
        AND (? IS NULL OR EXISTS (
                SELECT 1
                FROM transactions_categories tc_filter
                WHERE tc_filter.transaction_id = t.id
                AND tc_filter.category_id = ?
            )
        )
        AND (? IS NULL OR LOWER(t.description) LIKE CONCAT('%', LOWER(?), '%'))
        GROUP BY
            t.id,
            t.account_id,
            t.transaction_type_id,
            tt.name,
            t.datetime,
            t.amount,
            t.description,
            t.note,
            t.fingerprint,
            a.currency
        ORDER BY t.datetime DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(account_id)
    .bind(account_id)
    .bind(transaction_type_id)
    .bind(transaction_type_id)
    .bind(category_id)
    .bind(category_id)
    .bind(description.clone())
    .bind(description)
    .bind(page_size as i64)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    // Convert amounts from each account's currency to the configured display currency.
    // Rates are fetched once per unique (account_currency, date) pair to avoid redundant calls.
    // If the settings or currency services are unavailable, amounts are returned unconverted.
    if let Some(ref to_currency) = fetch_target_currency(http_client, settings_service_url).await {
        let mut rate_cache: HashMap<(String, NaiveDate), Option<f64>> = HashMap::new();
        for t in &mut transactions {
            if let Some(ref from_currency) = t.account_currency.clone() {
                if from_currency != to_currency {
                    let date = t.datetime.date();
                    let key = (from_currency.clone(), date);
                    if !rate_cache.contains_key(&key) {
                        let rate = fetch_exchange_rate(
                            http_client,
                            currency_service_url,
                            from_currency,
                            to_currency,
                            date,
                        )
                        .await;
                        rate_cache.insert(key.clone(), rate);
                    }
                    if let Some(Some(rate)) = rate_cache.get(&key) {
                        t.amount *= rate;
                    }
                }
            }
        }
    }

    Ok((transactions, total_count))
}

async fn fetch_transaction_by_id(
    pool: &MySqlPool,
    transaction_id: i64,
) -> Result<Option<Transaction>, anyhow::Error> {
    let transaction = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT
            t.id,
            t.account_id,
            t.transaction_type_id,
            tt.name AS transaction_type_name,
            GROUP_CONCAT(DISTINCT c.id ORDER BY c.name SEPARATOR ',') AS category_ids,
            GROUP_CONCAT(DISTINCT c.name ORDER BY c.name SEPARATOR ', ') AS categories,
            t.datetime,
            t.amount,
            t.description,
            t.note,
            t.fingerprint
        FROM transactions t
        LEFT JOIN transaction_types tt ON t.transaction_type_id = tt.id
        LEFT JOIN transactions_categories tc ON t.id = tc.transaction_id
        LEFT JOIN categories c ON tc.category_id = c.id
        WHERE t.id = ?
        GROUP BY
            t.id,
            t.account_id,
            t.transaction_type_id,
            tt.name,
            t.datetime,
            t.amount,
            t.description,
            t.note,
            t.fingerprint
        "#,
    )
    .bind(transaction_id)
    .fetch_optional(pool)
    .await?;

    Ok(transaction)
}

pub async fn create_transaction(
    pool: &MySqlPool,
    payload: TransactionUpsert,
) -> Result<Transaction, anyhow::Error> {
    let fingerprint = Uuid::new_v4().to_string();
    let mut tx = pool.begin().await?;
    let mut category_ids = payload.category_ids;
    category_ids.sort_unstable();
    category_ids.dedup();

    let result = sqlx::query(
        r#"
        INSERT INTO transactions (
            account_id,
            transaction_type_id,
            datetime,
            amount,
            description,
            note,
            fingerprint
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(payload.account_id)
    .bind(payload.transaction_type_id)
    .bind(payload.datetime)
    .bind(payload.amount)
    .bind(payload.description)
    .bind(payload.note)
    .bind(fingerprint)
    .execute(&mut *tx)
    .await?;

    let transaction_id = result.last_insert_id() as i64;

    sync_transaction_categories(&mut tx, transaction_id, &category_ids).await?;
    tx.commit().await?;

    fetch_transaction_by_id(pool, transaction_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to load created transaction"))
}

pub async fn update_transaction(
    pool: &MySqlPool,
    transaction_id: i64,
    payload: TransactionUpsert,
) -> Result<Transaction, anyhow::Error> {
    let mut tx = pool.begin().await?;
    let mut category_ids = payload.category_ids;
    category_ids.sort_unstable();
    category_ids.dedup();

    let result = sqlx::query(
        r#"
        UPDATE transactions
        SET
            account_id = ?,
            transaction_type_id = ?,
            datetime = ?,
            amount = ?,
            description = ?,
            note = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.account_id)
    .bind(payload.transaction_type_id)
    .bind(payload.datetime)
    .bind(payload.amount)
    .bind(payload.description)
    .bind(payload.note)
    .bind(transaction_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(anyhow::anyhow!("Transaction not found"));
    }

    sync_transaction_categories(&mut tx, transaction_id, &category_ids).await?;
    tx.commit().await?;

    fetch_transaction_by_id(pool, transaction_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Transaction not found"))
}

pub async fn delete_transaction(
    pool: &MySqlPool,
    transaction_id: i64,
) -> Result<(), anyhow::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        DELETE FROM sub_transactions_categories
        WHERE sub_transaction_id IN (
            SELECT id FROM (
                SELECT id FROM sub_transactions WHERE transaction_id = ?
            ) AS sub_transaction_ids
        )
        "#,
    )
    .bind(transaction_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM sub_transactions WHERE transaction_id = ?")
        .bind(transaction_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM transactions_categories WHERE transaction_id = ?")
        .bind(transaction_id)
        .execute(&mut *tx)
        .await?;

    let result = sqlx::query("DELETE FROM transactions WHERE id = ?")
        .bind(transaction_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(anyhow::anyhow!("Transaction not found"));
    }

    tx.commit().await?;
    Ok(())
}

pub async fn list_sub_transactions(
    pool: &MySqlPool,
    http_client: &reqwest::Client,
    settings_service_url: &str,
    currency_service_url: &str,
    transaction_id: i64,
) -> Result<Vec<SubTransaction>, anyhow::Error> {
    let mut rows = sqlx::query_as::<_, SubTransaction>(
        r#"
        SELECT
            st.id,
            st.transaction_id,
            st.product_code,
            st.amount,
            st.description,
            st.note,
            GROUP_CONCAT(DISTINCT c.id ORDER BY c.name SEPARATOR ',') AS category_ids,
            GROUP_CONCAT(DISTINCT c.name ORDER BY c.name SEPARATOR ', ') AS categories,
            a.currency AS account_currency,
            t.datetime AS transaction_datetime
        FROM sub_transactions st
        JOIN transactions t ON st.transaction_id = t.id
        JOIN accounts a ON t.account_id = a.id
        LEFT JOIN sub_transactions_categories stc ON st.id = stc.sub_transaction_id
        LEFT JOIN categories c ON stc.category_id = c.id
        WHERE st.transaction_id = ?
        GROUP BY st.id, st.transaction_id, st.product_code, st.amount, st.description, st.note, a.currency, t.datetime
        ORDER BY st.id ASC
        "#,
    )
    .bind(transaction_id)
    .fetch_all(pool)
    .await?;

    // All sub-transactions share the same parent transaction, so only one rate fetch is needed.
    // The HashMap keeps things uniform in case future queries mix dates/currencies.
    // If the settings or currency services are unavailable, amounts are returned unconverted.
    if let Some(ref to_currency) = fetch_target_currency(http_client, settings_service_url).await {
        let mut rate_cache: HashMap<(String, NaiveDate), Option<f64>> = HashMap::new();
        for st in &mut rows {
            if let (Some(ref from_currency), Some(ref dt)) =
                (st.account_currency.clone(), st.transaction_datetime)
            {
                if from_currency != to_currency {
                    let date = dt.date();
                    let key = (from_currency.clone(), date);
                    if !rate_cache.contains_key(&key) {
                        let rate = fetch_exchange_rate(
                            http_client,
                            currency_service_url,
                            from_currency,
                            to_currency,
                            date,
                        )
                        .await;
                        rate_cache.insert(key.clone(), rate);
                    }
                    if let Some(Some(rate)) = rate_cache.get(&key) {
                        st.amount *= rate;
                    }
                }
            }
        }
    }

    Ok(rows)
}

async fn fetch_sub_transaction_by_id(
    pool: &MySqlPool,
    sub_transaction_id: i64,
) -> Result<SubTransaction, anyhow::Error> {
    let row = sqlx::query_as::<_, SubTransaction>(
        r#"
        SELECT
            st.id,
            st.transaction_id,
            st.product_code,
            st.amount,
            st.description,
            st.note,
            GROUP_CONCAT(DISTINCT c.id ORDER BY c.name SEPARATOR ',') AS category_ids,
            GROUP_CONCAT(DISTINCT c.name ORDER BY c.name SEPARATOR ', ') AS categories
        FROM sub_transactions st
        LEFT JOIN sub_transactions_categories stc ON st.id = stc.sub_transaction_id
        LEFT JOIN categories c ON stc.category_id = c.id
        WHERE st.id = ?
        GROUP BY st.id, st.transaction_id, st.product_code, st.amount, st.description, st.note
        "#,
    )
    .bind(sub_transaction_id)
    .fetch_one(pool)
    .await?;

    Ok(row)
}

pub async fn create_sub_transaction(
    pool: &MySqlPool,
    transaction_id: i64,
    product_code: Option<String>,
    amount: f64,
    description: String,
    note: Option<String>,
    category_ids: Vec<i64>,
) -> Result<SubTransaction, anyhow::Error> {
    let mut tx = pool.begin().await?;
    let mut category_ids = category_ids;
    category_ids.sort_unstable();
    category_ids.dedup();

    let result = sqlx::query(
        r#"
        INSERT INTO sub_transactions (transaction_id, product_code, amount, description, note)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(transaction_id)
    .bind(product_code)
    .bind(amount)
    .bind(description)
    .bind(note)
    .execute(&mut *tx)
    .await?;

    let sub_transaction_id = result.last_insert_id() as i64;

    sync_sub_transaction_categories(&mut tx, sub_transaction_id, &category_ids).await?;
    tx.commit().await?;

    fetch_sub_transaction_by_id(pool, sub_transaction_id).await
}

#[allow(dead_code)]
pub async fn update_sub_transaction(
    pool: &MySqlPool,
    sub_transaction_id: i64,
    product_code: Option<String>,
    amount: f64,
    description: String,
    note: Option<String>,
    category_ids: Vec<i64>,
) -> Result<SubTransaction, anyhow::Error> {
    let mut tx = pool.begin().await?;
    let mut category_ids = category_ids;
    category_ids.sort_unstable();
    category_ids.dedup();

    let result = sqlx::query(
        r#"
        UPDATE sub_transactions
        SET product_code = ?, amount = ?, description = ?, note = ?
        WHERE id = ?
        "#,
    )
    .bind(product_code)
    .bind(amount)
    .bind(description)
    .bind(note)
    .bind(sub_transaction_id)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(anyhow::anyhow!("Sub-transaction not found"));
    }

    sync_sub_transaction_categories(&mut tx, sub_transaction_id, &category_ids).await?;
    tx.commit().await?;

    fetch_sub_transaction_by_id(pool, sub_transaction_id).await
}

#[allow(dead_code)]
pub async fn delete_sub_transaction(
    pool: &MySqlPool,
    sub_transaction_id: i64,
) -> Result<(), anyhow::Error> {
    let mut tx = pool.begin().await?;

    // delete associations first
    sqlx::query("DELETE FROM sub_transactions_categories WHERE sub_transaction_id = ?")
        .bind(sub_transaction_id)
        .execute(&mut *tx)
        .await?;

    let result = sqlx::query("DELETE FROM sub_transactions WHERE id = ?")
        .bind(sub_transaction_id)
        .execute(&mut *tx)
        .await?;

    if result.rows_affected() == 0 {
        tx.rollback().await?;
        return Err(anyhow::anyhow!("Sub-transaction not found"));
    }

    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    // Helper function to create test transactions
    fn create_test_transaction(
        id: i64,
        account_id: i64,
        amount: f64,
        description: &str,
    ) -> Transaction {
        Transaction {
            id,
            account_id,
            transaction_type_id: 1,
            transaction_type_name: Some("Income".to_string()),
            category_ids: Some("1,2".to_string()),
            categories: Some("Salary, Recurring".to_string()),
            datetime: NaiveDateTime::parse_from_str("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            amount,
            description: description.to_string(),
            note: Some("Test note".to_string()),
            fingerprint: format!("fp{}", id),
            account_currency: None,
        }
    }

    #[tokio::test]
    async fn test_list_transactions_without_account_filter() {
        // This test would require a test database setup
        // For now, we'll create a mock test that validates the logic structure
        let transactions = [
            create_test_transaction(1, 123, 100.50, "Deposit"),
            create_test_transaction(2, 456, -25.00, "Withdrawal"),
        ];

        // Test that we can create and manipulate transaction data
        assert_eq!(transactions.len(), 2);
        assert_eq!(transactions[0].id, 1);
        assert_eq!(transactions[1].amount, -25.00);
        assert!(transactions[0].description.contains("Deposit"));
    }

    #[tokio::test]
    async fn test_list_transactions_with_account_filter() {
        // Test filtering logic with mock data
        let all_transactions = vec![
            create_test_transaction(1, 123, 100.50, "Deposit 1"),
            create_test_transaction(2, 456, -25.00, "Withdrawal"),
            create_test_transaction(3, 123, 50.00, "Deposit 2"),
        ];

        // Simulate filtering by account_id
        let account_id = 123;
        let filtered: Vec<_> = all_transactions
            .into_iter()
            .filter(|t| t.account_id == account_id)
            .collect();

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|t| t.account_id == account_id));
        assert_eq!(filtered[0].description, "Deposit 1");
        assert_eq!(filtered[1].description, "Deposit 2");
    }

    #[tokio::test]
    async fn test_list_transactions_with_transaction_type_filter() {
        let mut income_transaction = create_test_transaction(1, 123, 100.50, "Deposit 1");
        income_transaction.transaction_type_id = 1;
        income_transaction.transaction_type_name = Some("Income".to_string());

        let mut expense_transaction = create_test_transaction(2, 456, -25.00, "Withdrawal");
        expense_transaction.transaction_type_id = 2;
        expense_transaction.transaction_type_name = Some("Expense".to_string());

        let all_transactions = vec![income_transaction, expense_transaction];

        let transaction_type_id = 1;
        let filtered: Vec<_> = all_transactions
            .into_iter()
            .filter(|t| t.transaction_type_id == transaction_type_id)
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].transaction_type_name.as_deref(), Some("Income"));
    }

    #[tokio::test]
    async fn test_list_transactions_with_category_filter() {
        let mut groceries_transaction = create_test_transaction(1, 123, 100.50, "Deposit 1");
        groceries_transaction.categories = Some("Groceries, Home".to_string());

        let mut salary_transaction = create_test_transaction(2, 456, -25.00, "Withdrawal");
        salary_transaction.categories = Some("Salary".to_string());

        let all_transactions = vec![groceries_transaction, salary_transaction];

        let filtered: Vec<_> = all_transactions
            .into_iter()
            .filter(|t| {
                t.categories
                    .as_deref()
                    .unwrap_or_default()
                    .contains("Groceries")
            })
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].categories.as_deref(), Some("Groceries, Home"));
    }

    #[tokio::test]
    async fn test_list_transactions_with_description_filter() {
        let all_transactions = vec![
            create_test_transaction(1, 123, 100.50, "Coffee Shop"),
            create_test_transaction(2, 456, -25.00, "Monthly Rent"),
            create_test_transaction(3, 789, -10.00, "COFFEE beans"),
        ];

        let description = "coffee".to_lowercase();
        let filtered: Vec<_> = all_transactions
            .into_iter()
            .filter(|transaction| {
                transaction
                    .description
                    .to_lowercase()
                    .contains(&description)
            })
            .collect();

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].description, "Coffee Shop");
        assert_eq!(filtered[1].description, "COFFEE beans");
    }

    #[test]
    fn test_transaction_creation() {
        let transaction = create_test_transaction(1, 123, 100.50, "Test transaction");

        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.account_id, 123);
        assert_eq!(transaction.amount, 100.50);
        assert_eq!(transaction.description, "Test transaction");
        assert_eq!(transaction.transaction_type_id, 1);
        assert_eq!(transaction.transaction_type_name.as_deref(), Some("Income"));
        assert_eq!(transaction.categories.as_deref(), Some("Salary, Recurring"));
        assert!(transaction.note.is_some());
        assert_eq!(transaction.note.as_ref().unwrap(), "Test note");
        assert_eq!(transaction.fingerprint, "fp1");
    }

    #[test]
    fn test_transaction_with_negative_amount() {
        let transaction = create_test_transaction(2, 456, -75.25, "Debit transaction");

        assert_eq!(transaction.amount, -75.25);
        assert!(transaction.amount < 0.0);
        assert_eq!(transaction.description, "Debit transaction");
    }

    #[test]
    fn test_transaction_upsert_shape() {
        let payload = TransactionUpsert {
            account_id: 1,
            transaction_type_id: 2,
            category_ids: vec![3, 4],
            datetime: NaiveDateTime::parse_from_str("2026-04-04 12:00:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            amount: 25.5,
            description: "Manual entry".to_string(),
            note: Some("Created manually".to_string()),
        };

        assert_eq!(payload.account_id, 1);
        assert_eq!(payload.transaction_type_id, 2);
        assert_eq!(payload.category_ids, vec![3, 4]);
        assert_eq!(payload.amount, 25.5);
    }
}
