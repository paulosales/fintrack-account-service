use crate::services::account_service;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use redis::aio::ConnectionManager;
use sqlx::MySqlPool;

pub async fn list_accounts(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
) -> impl IntoResponse {
    match account_service::list_accounts(&pool, &mut cache).await {
        Ok(accounts) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": accounts,
                "count": accounts.len()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch accounts: {}", e)
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    // Mock account for testing
    fn create_mock_account(id: i64, code: &str, name: &str) -> crate::models::accounts::Account {
        crate::models::accounts::Account {
            id,
            code: code.to_string(),
            name: name.to_string(),
            account_type_id: 1,
        }
    }

    #[test]
    fn test_create_mock_account() {
        let account = create_mock_account(1, "CHK-001", "Checking Account");
        assert_eq!(account.id, 1);
        assert_eq!(account.code, "CHK-001");
        assert_eq!(account.name, "Checking Account");
    }
}
