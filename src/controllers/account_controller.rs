use crate::services::account_service;
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use redis::aio::ConnectionManager;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountPayload {
    code: String,
    name: String,
    account_type_id: i64,
    currency: Option<String>,
}

fn map_payload(
    payload: AccountPayload,
) -> Result<crate::models::accounts::AccountUpsert, &'static str> {
    let code = payload.code.trim().to_string();
    let name = payload.name.trim().to_string();
    let currency = payload.currency.as_ref().map(|c| c.trim().to_string());

    if code.is_empty() || name.is_empty() {
        return Err("Account code and name are required");
    }

    if payload.account_type_id <= 0 {
        return Err("A valid account type is required");
    }

    Ok(crate::models::accounts::AccountUpsert {
        code,
        name,
        account_type_id: payload.account_type_id,
        currency: currency.filter(|c| !c.is_empty()),
    })
}

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

pub async fn create_account(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Json(payload): Json<AccountPayload>,
) -> impl IntoResponse {
    let payload = match map_payload(payload) {
        Ok(p) => p,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "success": false, "error": error })),
            )
                .into_response();
        }
    };

    match account_service::create_account(&pool, &mut cache, payload).await {
        Ok(account) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({ "success": true, "data": account })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to create account: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn update_account(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Path(account_id): Path<i64>,
    Json(payload): Json<AccountPayload>,
) -> impl IntoResponse {
    let payload = match map_payload(payload) {
        Ok(p) => p,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "success": false, "error": error })),
            )
                .into_response();
        }
    };

    match account_service::update_account(&pool, &mut cache, account_id, payload).await {
        Ok(account) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "data": account })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to update account: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn delete_account(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Path(account_id): Path<i64>,
) -> impl IntoResponse {
    match account_service::delete_account(&pool, &mut cache, account_id).await {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete account: {}", error)
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::map_payload;
    use crate::models::accounts::Account;

    #[test]
    fn test_create_mock_account() {
        let account = Account {
            id: 1,
            code: "CHK-001".to_string(),
            currency: Some("USD".to_string()),
            name: "Checking Account".to_string(),
            account_type_id: 1,
        };
        assert_eq!(account.id, 1);
        assert_eq!(account.code, "CHK-001");
        assert_eq!(account.name, "Checking Account");
    }

    #[test]
    fn test_map_payload_trims_fields() {
        let payload = super::AccountPayload {
            code: "  CHK-001  ".to_string(),
            name: "  Checking  ".to_string(),
            currency: Some("  USD  ".to_string()),
            account_type_id: 1,
        };
        let mapped = map_payload(payload).unwrap();
        assert_eq!(mapped.code, "CHK-001");
        assert_eq!(mapped.name, "Checking");
        assert_eq!(mapped.currency, Some("USD".to_string()));
    }

    #[test]
    fn test_map_payload_requires_code_and_name() {
        let payload = super::AccountPayload {
            code: "  ".to_string(),
            currency: Some("  USD  ".to_string()),
            name: "  Checking  ".to_string(),
            account_type_id: 1,
        };
        assert!(map_payload(payload).is_err());
    }
}
