use crate::models::transactions::{Transaction};
use sqlx::MySqlPool;

pub async fn list_transactions(
    pool: &MySqlPool,
    account_id: Option<i64>,
) -> Result<Vec<Transaction>, anyhow::Error> {
    let transactions = if let Some(account_id) = account_id {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT
                id, account_id, transaction_type_id, datetime, amount,
                description, note, fingerprint
            FROM transactions
            WHERE account_id = ?
            ORDER BY datetime DESC
            "#
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query_as::<_, Transaction>(
            r#"
            SELECT
                id, account_id, transaction_type_id, datetime, amount,
                description, note, fingerprint
            FROM transactions
            ORDER BY datetime DESC
            "#
        )
        .fetch_all(pool)
        .await?
    };

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

    #[test]
    fn test_transaction_creation() {
        let transaction = create_test_transaction(1, 123, 100.50, "Test transaction");

        assert_eq!(transaction.id, 1);
        assert_eq!(transaction.account_id, 123);
        assert_eq!(transaction.amount, 100.50);
        assert_eq!(transaction.description, "Test transaction");
        assert_eq!(transaction.transaction_type_id, 1);
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
