use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use redis::aio::ConnectionManager;
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::services::transaction_type_service;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionTypePayload {
    code: String,
    name: String,
}

fn map_payload(
    payload: TransactionTypePayload,
) -> Result<crate::models::transaction_types::TransactionTypeUpsert, &'static str> {
    let code = payload.code.trim().to_uppercase();
    let name = payload.name.trim().to_string();

    if code.is_empty() {
        return Err("Transaction type code is required");
    }

    if name.is_empty() {
        return Err("Transaction type name is required");
    }

    Ok(crate::models::transaction_types::TransactionTypeUpsert { code, name })
}

pub async fn list_transaction_types(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
) -> impl IntoResponse {
    match transaction_type_service::list_transaction_types(&pool, &mut cache).await {
        Ok(transaction_types) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": transaction_types,
                "count": transaction_types.len()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch transaction types: {}", e)
            })),
        )
            .into_response(),
    }
}

pub async fn create_transaction_type(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Json(payload): Json<TransactionTypePayload>,
) -> impl IntoResponse {
    let payload = match map_payload(payload) {
        Ok(payload) => payload,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": error
                })),
            )
                .into_response();
        }
    };

    match transaction_type_service::create_transaction_type(&pool, &mut cache, payload).await {
        Ok(transaction_type) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({
                "success": true,
                "data": transaction_type
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to create transaction type: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn update_transaction_type(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Path(transaction_type_id): Path<i64>,
    Json(payload): Json<TransactionTypePayload>,
) -> impl IntoResponse {
    let payload = match map_payload(payload) {
        Ok(payload) => payload,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": error
                })),
            )
                .into_response();
        }
    };

    match transaction_type_service::update_transaction_type(
        &pool,
        &mut cache,
        transaction_type_id,
        payload,
    )
    .await
    {
        Ok(transaction_type) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": transaction_type
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to update transaction type: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn delete_transaction_type(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Path(transaction_type_id): Path<i64>,
) -> impl IntoResponse {
    match transaction_type_service::delete_transaction_type(&pool, &mut cache, transaction_type_id)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete transaction type: {}", error)
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::map_payload;
    use crate::models::transaction_types::TransactionType;

    #[test]
    fn test_create_mock_transaction_type() {
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
    fn test_map_payload_trims_and_uppercases_code() {
        let payload = super::TransactionTypePayload {
            code: " payment ".to_string(),
            name: "  Payment  ".to_string(),
        };

        let mapped = map_payload(payload).unwrap();

        assert_eq!(mapped.code, "PAYMENT");
        assert_eq!(mapped.name, "Payment");
    }

    #[test]
    fn test_map_payload_rejects_empty_code() {
        let payload = super::TransactionTypePayload {
            code: "   ".to_string(),
            name: "Payment".to_string(),
        };

        assert_eq!(
            map_payload(payload).unwrap_err(),
            "Transaction type code is required"
        );
    }

    #[test]
    fn test_map_payload_rejects_empty_name() {
        let payload = super::TransactionTypePayload {
            code: "PAYMENT".to_string(),
            name: "   ".to_string(),
        };

        assert_eq!(
            map_payload(payload).unwrap_err(),
            "Transaction type name is required"
        );
    }
}
