use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub id: i64,
    pub account_id: i64,
    pub transaction_type_id: i64,
    pub transaction_type_name: Option<String>,
    pub category_ids: Option<String>,
    pub categories: Option<String>,
    pub datetime: NaiveDateTime,
    pub amount: f64,
    pub description: String,
    pub note: Option<String>,
    pub fingerprint: String,
    /// Account currency used for conversion; not included in API responses.
    #[serde(skip_serializing)]
    #[sqlx(default)]
    pub account_currency: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_transaction_serialization() {
        let datetime =
            NaiveDateTime::parse_from_str("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let transaction = Transaction {
            id: 1,
            account_id: 123,
            transaction_type_id: 2,
            transaction_type_name: Some("Expense".to_string()),
            category_ids: Some("1,2".to_string()),
            categories: Some("Groceries, Home".to_string()),
            datetime,
            amount: 150.50,
            description: "Grocery shopping".to_string(),
            note: Some("Weekly groceries".to_string()),
            fingerprint: "abc123def456".to_string(),
            account_currency: None,
        };

        let json = serde_json::to_string(&transaction).unwrap();
        let deserialized: Transaction = serde_json::from_str(&json).unwrap();

        assert_eq!(transaction.id, deserialized.id);
        assert_eq!(transaction.account_id, deserialized.account_id);
        assert_eq!(
            transaction.transaction_type_id,
            deserialized.transaction_type_id
        );
        assert_eq!(
            transaction.transaction_type_name,
            deserialized.transaction_type_name
        );
        assert_eq!(transaction.category_ids, deserialized.category_ids);
        assert_eq!(transaction.categories, deserialized.categories);
        assert_eq!(transaction.datetime, deserialized.datetime);
        assert_eq!(transaction.amount, deserialized.amount);
        assert_eq!(transaction.description, deserialized.description);
        assert_eq!(transaction.note, deserialized.note);
        assert_eq!(transaction.fingerprint, deserialized.fingerprint);
    }

    #[test]
    fn test_transaction_serialization_without_note() {
        let datetime =
            NaiveDateTime::parse_from_str("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let transaction = Transaction {
            id: 2,
            account_id: 456,
            transaction_type_id: 1,
            transaction_type_name: Some("Income".to_string()),
            category_ids: None,
            categories: None,
            datetime,
            amount: -50.00,
            description: "ATM withdrawal".to_string(),
            note: None,
            fingerprint: "def789ghi012".to_string(),
            account_currency: None,
        };

        let json = serde_json::to_string(&transaction).unwrap();
        let deserialized: Transaction = serde_json::from_str(&json).unwrap();

        assert_eq!(transaction.id, deserialized.id);
        assert_eq!(transaction.note, deserialized.note);
        assert!(deserialized.note.is_none());
    }

    #[test]
    fn test_transaction_debug_format() {
        let datetime =
            NaiveDateTime::parse_from_str("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let transaction = Transaction {
            id: 1,
            account_id: 123,
            transaction_type_id: 2,
            transaction_type_name: Some("Expense".to_string()),
            category_ids: Some("1".to_string()),
            categories: Some("Groceries".to_string()),
            datetime,
            amount: 100.00,
            description: "Test transaction".to_string(),
            note: Some("Test note".to_string()),
            fingerprint: "test123".to_string(),
            account_currency: None,
        };

        let debug_str = format!("{:?}", transaction);
        assert!(debug_str.contains("Transaction"));
        assert!(debug_str.contains("id: 1"));
        assert!(debug_str.contains("amount: 100.0"));
        assert!(debug_str.contains("description: \"Test transaction\""));
    }
}
