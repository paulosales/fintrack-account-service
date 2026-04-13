use anyhow::{anyhow, bail};
use redis::aio::ConnectionManager;

use crate::models::account_types::{AccountType, AccountTypeUpsert};
use sqlx::MySqlPool;

const CACHE_KEY: &str = "account_types:all";

async fn get_account_type_by_id(
    pool: &MySqlPool,
    account_type_id: i64,
) -> Result<AccountType, anyhow::Error> {
    let account_type = sqlx::query_as::<_, AccountType>(
        r#"
        SELECT id, code, name
        FROM account_types
        WHERE id = ?
        "#,
    )
    .bind(account_type_id)
    .fetch_optional(pool)
    .await?;

    account_type.ok_or_else(|| anyhow!("Account type {} not found", account_type_id))
}

pub async fn list_account_types(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
) -> Result<Vec<AccountType>, anyhow::Error> {
    if let Some(cached) = crate::cache::get(cache, CACHE_KEY).await {
        if let Ok(account_types) = serde_json::from_str::<Vec<AccountType>>(&cached) {
            return Ok(account_types);
        }
    }

    let account_types = sqlx::query_as::<_, AccountType>(
        r#"
        SELECT id, code, name
        FROM account_types
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    if let Ok(json) = serde_json::to_string(&account_types) {
        crate::cache::set(cache, CACHE_KEY, &json).await;
    }

    Ok(account_types)
}

pub async fn create_account_type(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    payload: AccountTypeUpsert,
) -> Result<AccountType, anyhow::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO account_types (code, name)
        VALUES (?, ?)
        "#,
    )
    .bind(payload.code)
    .bind(payload.name)
    .execute(pool)
    .await?;

    crate::cache::del(cache, CACHE_KEY).await;

    get_account_type_by_id(pool, result.last_insert_id() as i64).await
}

pub async fn update_account_type(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    account_type_id: i64,
    payload: AccountTypeUpsert,
) -> Result<AccountType, anyhow::Error> {
    let result = sqlx::query(
        r#"
        UPDATE account_types
        SET code = ?, name = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.code)
    .bind(payload.name)
    .bind(account_type_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        bail!("Account type {} not found", account_type_id);
    }

    crate::cache::del(cache, CACHE_KEY).await;

    get_account_type_by_id(pool, account_type_id).await
}

pub async fn delete_account_type(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    account_type_id: i64,
) -> Result<(), anyhow::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM account_types
        WHERE id = ?
        "#,
    )
    .bind(account_type_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        bail!("Account type {} not found", account_type_id);
    }

    crate::cache::del(cache, CACHE_KEY).await;

    Ok(())
}
