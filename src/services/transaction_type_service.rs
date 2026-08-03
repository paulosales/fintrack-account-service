use crate::models::transaction_types::{TransactionType, TransactionTypeUpsert};
use anyhow::{anyhow, bail};
use redis::aio::ConnectionManager;
use sqlx::MySqlPool;

const CACHE_KEY: &str = "transaction_types:all";

async fn get_transaction_type_by_id(
    pool: &MySqlPool,
    transaction_type_id: i64,
) -> Result<TransactionType, anyhow::Error> {
    let transaction_type = sqlx::query_as::<_, TransactionType>(
        r#"
        SELECT id, code, name
        FROM transaction_types
        WHERE id = ?
        "#,
    )
    .bind(transaction_type_id)
    .fetch_optional(pool)
    .await?;

    transaction_type.ok_or_else(|| anyhow!("Transaction type {} not found", transaction_type_id))
}

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

pub async fn create_transaction_type(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    payload: TransactionTypeUpsert,
) -> Result<TransactionType, anyhow::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO transaction_types (code, name)
        VALUES (?, ?)
        "#,
    )
    .bind(payload.code)
    .bind(payload.name)
    .execute(pool)
    .await?;

    crate::cache::del(cache, CACHE_KEY).await;

    get_transaction_type_by_id(pool, result.last_insert_id() as i64).await
}

pub async fn update_transaction_type(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    transaction_type_id: i64,
    payload: TransactionTypeUpsert,
) -> Result<TransactionType, anyhow::Error> {
    let result = sqlx::query(
        r#"
        UPDATE transaction_types
        SET code = ?, name = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.code)
    .bind(payload.name)
    .bind(transaction_type_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        bail!("Transaction type {} not found", transaction_type_id);
    }

    crate::cache::del(cache, CACHE_KEY).await;

    get_transaction_type_by_id(pool, transaction_type_id).await
}

pub async fn delete_transaction_type(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    transaction_type_id: i64,
) -> Result<(), anyhow::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM transaction_types
        WHERE id = ?
        "#,
    )
    .bind(transaction_type_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        bail!("Transaction type {} not found", transaction_type_id);
    }

    crate::cache::del(cache, CACHE_KEY).await;

    Ok(())
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

    #[test]
    fn test_transaction_type_upsert_model() {
        let payload = TransactionTypeUpsert {
            code: "PAYMENT".to_string(),
            name: "Payment".to_string(),
        };

        assert_eq!(payload.code, "PAYMENT");
        assert_eq!(payload.name, "Payment");
    }
}
