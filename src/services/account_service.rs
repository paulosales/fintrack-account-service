use crate::models::accounts::Account;
use sqlx::MySqlPool;

pub async fn list_accounts(pool: &MySqlPool) -> Result<Vec<Account>, anyhow::Error> {
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

    Ok(accounts)
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
