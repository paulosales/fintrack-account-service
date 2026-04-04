use crate::models::transaction_types::TransactionType;
use sqlx::MySqlPool;

pub async fn list_transaction_types(pool: &MySqlPool) -> Result<Vec<TransactionType>, anyhow::Error> {
    let transaction_types = sqlx::query_as::<_, TransactionType>(
        r#"
        SELECT id, code, name
        FROM transaction_types
        ORDER BY name ASC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(transaction_types)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_type_model() {
        let transaction_type = TransactionType {
            id: 1,
            code: "INCOME".to_string(),
            name: "Income".to_string(),
        };

        assert_eq!(transaction_type.id, 1);
        assert_eq!(transaction_type.code, "INCOME");
        assert_eq!(transaction_type.name, "Income");
    }
}