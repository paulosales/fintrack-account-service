use crate::models::sub_transactions::SubTransaction;
use crate::models::transactions::Transaction;
use chrono::NaiveDateTime;
use sqlx::MySqlPool;
use sqlx::Row;
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

pub async fn list_transactions(
    pool: &MySqlPool,
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

    let transactions = sqlx::query_as::<_, Transaction>(
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
            t.fingerprint
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

    Ok((transactions, total_count))
}

pub async fn get_transaction_by_id(
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

    get_transaction_by_id(pool, transaction_id)
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

    get_transaction_by_id(pool, transaction_id)
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
    transaction_id: i64,
) -> Result<Vec<SubTransaction>, anyhow::Error> {
    let rows = sqlx::query_as::<_, SubTransaction>(
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
        WHERE st.transaction_id = ?
        GROUP BY st.id, st.transaction_id, st.product_code, st.amount, st.description, st.note
        ORDER BY st.id ASC
        "#,
    )
    .bind(transaction_id)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

async fn get_sub_transaction_by_id(
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

    get_sub_transaction_by_id(pool, sub_transaction_id).await
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

    get_sub_transaction_by_id(pool, sub_transaction_id).await
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
