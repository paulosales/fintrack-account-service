use axum::{extract::State, http::StatusCode, response::IntoResponse};
use sqlx::MySqlPool;

use crate::services::transaction_type_service;

pub async fn list_transaction_types(State(pool): State<MySqlPool>) -> impl IntoResponse {
    match transaction_type_service::list_transaction_types(&pool).await {
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

#[cfg(test)]
mod tests {
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
}
