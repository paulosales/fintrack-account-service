use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SubTransaction {
    pub id: i64,
    pub transaction_id: i64,
    pub product_code: Option<String>,
    pub amount: f64,
    pub description: String,
    pub note: Option<String>,
    pub category_ids: Option<String>,
    pub categories: Option<String>,
    /// Account currency fetched via JOIN; not included in API responses.
    #[serde(skip_serializing)]
    #[sqlx(default)]
    pub account_currency: Option<String>,
    /// Parent transaction datetime used as the conversion date; not included in API responses.
    #[serde(skip_serializing)]
    #[sqlx(default)]
    pub transaction_datetime: Option<NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sub_transaction() {
        let st = SubTransaction {
            id: 1,
            transaction_id: 2,
            product_code: Some("P1".to_string()),
            amount: 5.0,
            description: "Item".to_string(),
            note: None,
            category_ids: Some("1,2".to_string()),
            categories: Some("Groceries, Home".to_string()),
            account_currency: None,
            transaction_datetime: None,
        };

        assert_eq!(st.id, 1);
        assert_eq!(st.transaction_id, 2);
        assert_eq!(st.product_code.as_deref(), Some("P1"));
        assert_eq!(st.amount, 5.0);
        assert_eq!(st.description, "Item");
        assert_eq!(st.category_ids.as_deref(), Some("1,2"));
        assert_eq!(st.categories.as_deref(), Some("Groceries, Home"));
    }
}
