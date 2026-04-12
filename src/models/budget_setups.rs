use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BudgetSetupRecord {
    pub id: i64,
    pub account_id: i64,
    pub account_code: String,
    pub account_name: String,
    pub date: NaiveDate,
    pub is_repeatle: bool,
    pub repeat_frequency: Option<String>,
    pub end_date: Option<NaiveDate>,
    pub description: String,
    pub amount: f64,
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_setup_record_serialization() {
        let record = BudgetSetupRecord {
            id: 1,
            account_id: 2,
            account_code: "CHK".to_string(),
            account_name: "Checking".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 4, 16).unwrap(),
            is_repeatle: true,
            repeat_frequency: Some("MONTHLY".to_string()),
            end_date: Some(NaiveDate::from_ymd_opt(2026, 12, 16).unwrap()),
            description: "Insurance".to_string(),
            amount: -120.0,
            note: Some("Monthly bill".to_string()),
        };

        let json = serde_json::to_value(&record).unwrap();

        assert_eq!(json["accountCode"], "CHK");
        assert_eq!(json["isRepeatle"], true);
        assert_eq!(json["repeatFrequency"], "MONTHLY");
    }
}
