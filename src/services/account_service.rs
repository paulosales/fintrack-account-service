use crate::models::accounts::{Account, AccountUpsert};
use anyhow::{anyhow, bail};
use redis::aio::ConnectionManager;
use sqlx::MySqlPool;

const CACHE_KEY: &str = "accounts:all";

async fn get_account_by_id(pool: &MySqlPool, account_id: i64) -> Result<Account, anyhow::Error> {
    let account = sqlx::query_as::<_, Account>(
        r#"
        SELECT id, code, name, account_type_id
        FROM accounts
        WHERE id = ?
        "#,
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;

    account.ok_or_else(|| anyhow!("Account {} not found", account_id))
}

pub async fn list_accounts(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
) -> Result<Vec<Account>, anyhow::Error> {
    if let Some(cached) = crate::cache::get(cache, CACHE_KEY).await {
        if let Ok(accounts) = serde_json::from_str::<Vec<Account>>(&cached) {
            return Ok(accounts);
        }
    }

    let accounts = sqlx::query_as::<_, Account>(
        r#"
        SELECT
            id, code, name, account_type_id
        FROM accounts
        ORDER BY code ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    if let Ok(json) = serde_json::to_string(&accounts) {
        crate::cache::set(cache, CACHE_KEY, &json).await;
    }

    Ok(accounts)
}

pub async fn create_account(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    payload: AccountUpsert,
) -> Result<Account, anyhow::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO accounts (code, name, account_type_id)
        VALUES (?, ?, ?)
        "#,
    )
    .bind(&payload.code)
    .bind(&payload.name)
    .bind(payload.account_type_id)
    .execute(pool)
    .await?;

    crate::cache::del(cache, CACHE_KEY).await;

    get_account_by_id(pool, result.last_insert_id() as i64).await
}

pub async fn update_account(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    account_id: i64,
    payload: AccountUpsert,
) -> Result<Account, anyhow::Error> {
    let result = sqlx::query(
        r#"
        UPDATE accounts
        SET code = ?, name = ?, account_type_id = ?
        WHERE id = ?
        "#,
    )
    .bind(&payload.code)
    .bind(&payload.name)
    .bind(payload.account_type_id)
    .bind(account_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        bail!("Account {} not found", account_id);
    }

    crate::cache::del(cache, CACHE_KEY).await;

    get_account_by_id(pool, account_id).await
}

pub async fn delete_account(
    pool: &MySqlPool,
    cache: &mut ConnectionManager,
    account_id: i64,
) -> Result<(), anyhow::Error> {
    let result = sqlx::query(
        r#"
        DELETE FROM accounts
        WHERE id = ?
        "#,
    )
    .bind(account_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        bail!("Account {} not found", account_id);
    }

    crate::cache::del(cache, CACHE_KEY).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create test accounts
    fn create_test_account(id: i64, code: &str, name: &str, account_type_id: i64) -> Account {
        Account {
            id,
            code: code.to_string(),
            name: name.to_string(),
            account_type_id,
        }
    }

    #[test]
    fn test_create_test_account() {
        let account = create_test_account(1, "CHK-001", "Checking Account", 1);
        assert_eq!(account.id, 1);
        assert_eq!(account.code, "CHK-001");
        assert_eq!(account.name, "Checking Account");
        assert_eq!(account.account_type_id, 1);
    }
}
