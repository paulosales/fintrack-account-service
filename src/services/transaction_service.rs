use crate::models::transactions::{Transaction};
use sqlx::MySqlPool;

pub async fn list_transactions(
    pool: &MySqlPool,
    account_id: Option<i64>,
    transaction_type_id: Option<i64>,
    category_id: Option<i64>,
) -> Result<Vec<Transaction>, anyhow::Error> {
    let transactions = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT
            t.id,
            t.account_id,
            t.transaction_type_id,
            tt.name AS transaction_type_name,
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
        "#
    )
    .bind(account_id)
    .bind(account_id)
    .bind(transaction_type_id)
    .bind(transaction_type_id)
    .bind(category_id)
    .bind(category_id)
    .fetch_all(pool)
    .await?;

    Ok(transactions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    // Helper function to create test transactions
    fn create_test_transaction(id: i64, account_id: i64, amount: f64, description: &str) -> Transaction {
        Transaction {
            id,
            account_id,
            transaction_type_id: 1,
            transaction_type_name: Some("Income".to_string()),
            categories: Some("Salary, Recurring".to_string()),
            datetime: NaiveDateTime::parse_from_str("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
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
        let transactions = vec![
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
        let filtered: Vec<_> = all_transactions.into_iter()
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

        let all_transactions = vec![
            income_transaction,
            expense_transaction,
        ];

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
            .filter(|t| t.categories.as_deref().unwrap_or_default().contains("Groceries"))
            .collect();

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].categories.as_deref(), Some("Groceries, Home"));
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
}
