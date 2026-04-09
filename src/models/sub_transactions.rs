use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct SubTransaction {
    pub id: i64,
    pub transaction_id: i64,
    pub product_code: Option<String>,
    pub amount: f64,
    pub description: String,
    pub note: Option<String>,
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
        };

        assert_eq!(st.id, 1);
        assert_eq!(st.transaction_id, 2);
        assert_eq!(st.product_code.as_deref(), Some("P1"));
        assert_eq!(st.amount, 5.0);
        assert_eq!(st.description, "Item");
    }
}
