use crate::services::account_type_service;
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
pub struct AccountTypePayload {
    code: String,
    name: String,
}

fn map_payload(
    payload: AccountTypePayload,
) -> Result<crate::models::account_types::AccountTypeUpsert, &'static str> {
    let code = payload.code.trim().to_string();
    let name = payload.name.trim().to_string();

    if code.is_empty() || name.is_empty() {
        return Err("Account type code and name are required");
    }

    Ok(crate::models::account_types::AccountTypeUpsert { code, name })
}

pub async fn list_account_types(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
) -> impl IntoResponse {
    match account_type_service::list_account_types(&pool, &mut cache).await {
        Ok(account_types) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": account_types,
                "count": account_types.len()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch account types: {}", e)
            })),
        )
            .into_response(),
    }
}

pub async fn create_account_type(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Json(payload): Json<AccountTypePayload>,
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

    match account_type_service::create_account_type(&pool, &mut cache, payload).await {
        Ok(account_type) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({ "success": true, "data": account_type })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to create account type: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn update_account_type(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Path(account_type_id): Path<i64>,
    Json(payload): Json<AccountTypePayload>,
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

    match account_type_service::update_account_type(&pool, &mut cache, account_type_id, payload)
        .await
    {
        Ok(account_type) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "data": account_type })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to update account type: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn delete_account_type(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Path(account_type_id): Path<i64>,
) -> impl IntoResponse {
    match account_type_service::delete_account_type(&pool, &mut cache, account_type_id).await {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete account type: {}", error)
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::map_payload;
    use crate::models::account_types::AccountType;

    #[test]
    fn test_create_mock_account_type() {
        let account_type = AccountType {
            id: 1,
            code: "CHECKING".to_string(),
            name: "Checking".to_string(),
        };
        assert_eq!(account_type.id, 1);
        assert_eq!(account_type.code, "CHECKING");
        assert_eq!(account_type.name, "Checking");
    }

    #[test]
    fn test_map_payload_trims_fields() {
        let payload = super::AccountTypePayload {
            code: "  CHECKING  ".to_string(),
            name: "  Checking  ".to_string(),
        };
        let mapped = map_payload(payload).unwrap();
        assert_eq!(mapped.code, "CHECKING");
        assert_eq!(mapped.name, "Checking");
    }

    #[test]
    fn test_map_payload_requires_code_and_name() {
        let payload = super::AccountTypePayload {
            code: "  ".to_string(),
            name: "Checking".to_string(),
        };
        assert!(map_payload(payload).is_err());
    }
}
