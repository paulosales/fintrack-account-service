use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCategoryTotal {
    pub year: i32,
    pub month: i32,
    pub month_label: String,
    pub category_id: i64,
    pub category: String,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCategoryTotalDetail {
    pub id: i64,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub year: i32,
    pub month: i32,
    pub month_label: String,
    pub description: String,
    pub datetime: NaiveDateTime,
    pub note: String,
    pub category_id: i64,
    pub category: String,
    pub amount: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_category_total_serialization() {
        let total = TransactionCategoryTotal {
            year: 2026,
            month: 4,
            month_label: "2026-04".to_string(),
            category_id: 7,
            category: "Groceries".to_string(),
            total_amount: -123.45,
        };

        let json = serde_json::to_string(&total).unwrap();
        let deserialized: TransactionCategoryTotal = serde_json::from_str(&json).unwrap();

        assert_eq!(total.month_label, deserialized.month_label);
        assert_eq!(total.category_id, deserialized.category_id);
        assert_eq!(total.total_amount, deserialized.total_amount);
    }

    #[test]
    fn test_transaction_category_total_detail_serialization() {
        let detail = TransactionCategoryTotalDetail {
            id: 10,
            entry_type: "transaction".to_string(),
            year: 2026,
            month: 4,
            month_label: "2026-04".to_string(),
            description: "Market".to_string(),
            datetime: NaiveDateTime::parse_from_str("2026-04-03 13:45:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            note: "weekly".to_string(),
            category_id: 7,
            category: "Groceries".to_string(),
            amount: -50.0,
        };

        let json = serde_json::to_value(&detail).unwrap();

        assert_eq!(json["type"], "transaction");
        assert_eq!(json["monthLabel"], "2026-04");
        assert_eq!(json["datetime"], "2026-04-03T13:45:00");
        assert_eq!(json["categoryId"], 7);
    }
}
