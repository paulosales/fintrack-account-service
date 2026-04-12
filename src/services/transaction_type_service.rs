use crate::models::transaction_types::TransactionType;
use redis::aio::ConnectionManager;
use sqlx::MySqlPool;

const CACHE_KEY: &str = "transaction_types:all";

pub async fn list_transaction_types(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
) -> Result<Vec<TransactionType>, anyhow::Error> {
    if let Some(cached) = crate::cache::get(cache, CACHE_KEY).await {
        if let Ok(types) = serde_json::from_str::<Vec<TransactionType>>(&cached) {
            return Ok(types);
        }
    }

    let transaction_types = sqlx::query_as::<_, TransactionType>(
        r#"
        SELECT id, code, name
        FROM transaction_types
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    if let Ok(json) = serde_json::to_string(&transaction_types) {
        crate::cache::set(cache, CACHE_KEY, &json).await;
    }

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
