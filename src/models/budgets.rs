use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BudgetMonthTotal {
    pub year: i32,
    pub month: i32,
    pub month_label: String,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BudgetRecord {
    pub id: i64,
    pub budget_setup_id: i64,
    pub account_id: i64,
    pub account_code: String,
    pub account_name: String,
    pub date: NaiveDate,
    pub amount: f64,
    pub description: String,
    pub processed: bool,
    pub note: Option<String>,
    pub is_repeatle: bool,
    pub repeat_frequency: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_month_total_serialization() {
        let total = BudgetMonthTotal {
            year: 2026,
            month: 4,
            month_label: "2026-04".to_string(),
            total_amount: -123.45,
        };

        let json = serde_json::to_value(&total).unwrap();

        assert_eq!(json["monthLabel"], "2026-04");
        assert_eq!(json["totalAmount"], -123.45);
    }

    #[test]
    fn test_budget_record_serialization() {
        let record = BudgetRecord {
            id: 1,
            budget_setup_id: 10,
            account_id: 2,
            account_code: "CHK".to_string(),
            account_name: "Checking".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 4, 16).unwrap(),
            amount: -238.58,
            description: "AUTO INSURANCE".to_string(),
            processed: false,
            note: Some("Monthly".to_string()),
            is_repeatle: true,
            repeat_frequency: Some("MONTHLY".to_string()),
        };

        let json = serde_json::to_value(&record).unwrap();

        assert_eq!(json["budgetSetupId"], 10);
        assert_eq!(json["accountCode"], "CHK");
        assert_eq!(json["processed"], false);
        assert_eq!(json["isRepeatle"], true);
    }
}
