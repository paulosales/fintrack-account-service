use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TransactionType {
    pub id: i64,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct TransactionTypeUpsert {
    pub code: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_type_serialization() {
        let transaction_type = TransactionType {
            id: 1,
            code: "INCOME".to_string(),
            name: "Income".to_string(),
        };

        let json = serde_json::to_string(&transaction_type).unwrap();
        let deserialized: TransactionType = serde_json::from_str(&json).unwrap();

        assert_eq!(transaction_type.id, deserialized.id);
        assert_eq!(transaction_type.code, deserialized.code);
        assert_eq!(transaction_type.name, deserialized.name);
    }

    #[test]
    fn test_transaction_type_upsert() {
        let payload = TransactionTypeUpsert {
            code: "PAYMENT".to_string(),
            name: "Payment".to_string(),
        };

        assert_eq!(payload.code, "PAYMENT");
        assert_eq!(payload.name, "Payment");
    }
}
