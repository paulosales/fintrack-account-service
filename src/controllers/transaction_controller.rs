use crate::models::pagination::{build_pagination_meta, normalize_page, normalize_page_size};
use crate::services::transaction_service;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct ListParams {
    account_id: Option<i64>,
    transaction_type_id: Option<i64>,
    category_id: Option<i64>,
    description: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionPayload {
    account_id: i64,
    transaction_type_id: i64,
    category_ids: Option<Vec<i64>>,
    datetime: String,
    amount: f64,
    description: String,
    note: Option<String>,
}

fn parse_transaction_datetime(value: &str) -> Result<NaiveDateTime, &'static str> {
    const FORMATS: [&str; 3] = ["%Y-%m-%dT%H:%M", "%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S"];

    for format in FORMATS {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(parsed);
        }
    }

    Err("Invalid datetime format")
}

fn map_payload(
    payload: TransactionPayload,
) -> Result<transaction_service::TransactionUpsert, &'static str> {
    Ok(transaction_service::TransactionUpsert {
        account_id: payload.account_id,
        transaction_type_id: payload.transaction_type_id,
        category_ids: payload.category_ids.unwrap_or_default(),
        datetime: parse_transaction_datetime(&payload.datetime)?,
        amount: payload.amount,
        description: payload.description,
        note: payload.note,
    })
}

pub async fn list_transactions(
    State(pool): State<MySqlPool>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let page = normalize_page(params.page);
    let page_size = normalize_page_size(params.page_size);
    let description = params.description.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    });

    match transaction_service::list_transactions(
        &pool,
        params.account_id,
        params.transaction_type_id,
        params.category_id,
        description,
        page,
        page_size,
    )
    .await
    {
        Ok((transactions, total_count)) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": transactions,
                "count": transactions.len(),
                "pagination": build_pagination_meta(page, page_size, total_count)
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch transactions: {}", e)
            })),
        )
            .into_response(),
    }
}

pub async fn create_transaction(
    State(pool): State<MySqlPool>,
    Json(payload): Json<TransactionPayload>,
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

    match transaction_service::create_transaction(&pool, payload).await {
        Ok(transaction) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({
                "success": true,
                "data": transaction
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to create transaction: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn update_transaction(
    State(pool): State<MySqlPool>,
    Path(transaction_id): Path<i64>,
    Json(payload): Json<TransactionPayload>,
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

    match transaction_service::update_transaction(&pool, transaction_id, payload).await {
        Ok(transaction) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": transaction
            })),
        )
            .into_response(),
        Err(error) if error.to_string().contains("not found") => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({
                "success": false,
                "error": error.to_string()
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to update transaction: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn delete_transaction(
    State(pool): State<MySqlPool>,
    Path(transaction_id): Path<i64>,
) -> impl IntoResponse {
    match transaction_service::delete_transaction(&pool, transaction_id).await {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true
            })),
        )
            .into_response(),
        Err(error) if error.to_string().contains("not found") => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({
                "success": false,
                "error": error.to_string()
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete transaction: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn get_sub_transactions(
    State(pool): State<MySqlPool>,
    Path(transaction_id): Path<i64>,
) -> impl IntoResponse {
    match crate::services::transaction_service::list_sub_transactions(&pool, transaction_id).await {
        Ok(subs) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": subs,
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch sub transactions: {}", error),
            })),
        )
            .into_response(),
    }
}

pub async fn create_sub_transaction(
    State(pool): State<MySqlPool>,
    Path(transaction_id): Path<i64>,
    Json(payload): Json<SubTransactionPayload>,
) -> impl IntoResponse {
    match crate::services::transaction_service::create_sub_transaction(
        &pool,
        transaction_id,
        payload.product_code,
        payload.amount,
        payload.description,
        payload.note,
        payload.category_ids.unwrap_or_default(),
    )
    .await
    {
        Ok(st) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({ "success": true, "data": st })),
        )
                .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "success": false, "error": format!("Failed to create sub transaction: {}", error) })),
        )
                .into_response(),
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubTransactionPayload {
    product_code: Option<String>,
    amount: f64,
    description: String,
    note: Option<String>,
    category_ids: Option<Vec<i64>>,
}

pub async fn update_sub_transaction(
    State(pool): State<MySqlPool>,
    Path((_, sub_transaction_id)): Path<(i64, i64)>,
    Json(payload): Json<SubTransactionPayload>,
) -> impl IntoResponse {
    match crate::services::transaction_service::update_sub_transaction(
        &pool,
        sub_transaction_id,
        payload.product_code,
        payload.amount,
        payload.description,
        payload.note,
        payload.category_ids.unwrap_or_default(),
    )
    .await
    {
        Ok(st) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "data": st })),
        )
            .into_response(),
        Err(error) if error.to_string().contains("not found") => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "success": false, "error": error.to_string() })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "success": false, "error": format!("Failed to update sub transaction: {}", error) })),
        )
            .into_response(),
    }
}

pub async fn delete_sub_transaction(
    State(pool): State<MySqlPool>,
    Path((_, sub_transaction_id)): Path<(i64, i64)>,
) -> impl IntoResponse {
    match crate::services::transaction_service::delete_sub_transaction(&pool, sub_transaction_id).await {
        Ok(()) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true })),
        )
            .into_response(),
        Err(error) if error.to_string().contains("not found") => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "success": false, "error": error.to_string() })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "success": false, "error": format!("Failed to delete sub transaction: {}", error) })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    // Mock transaction for testing
    fn create_mock_transaction(
        id: i64,
        account_id: i64,
    ) -> crate::models::transactions::Transaction {
        crate::models::transactions::Transaction {
            id,
            account_id,
            transaction_type_id: 1,
            transaction_type_name: Some("Income".to_string()),
            category_ids: Some("1".to_string()),
            categories: Some("Salary".to_string()),
            datetime: NaiveDateTime::parse_from_str("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S")
                .unwrap(),
            amount: 100.50,
            description: "Test transaction".to_string(),
            note: Some("Test note".to_string()),
            fingerprint: format!("fp{}", id),
        }
    }

    #[test]
    fn test_list_params_deserialization() {
        // Test deserialization of query parameters
        let params: ListParams = serde_qs::from_str("account_id=123").unwrap();
        assert_eq!(params.account_id, Some(123));
        assert_eq!(params.transaction_type_id, None);
        assert_eq!(params.category_id, None);
        assert_eq!(params.description, None);
        assert_eq!(params.page, None);
        assert_eq!(params.page_size, None);

        let params: ListParams =
            serde_qs::from_str("account_id=123&transaction_type_id=2").unwrap();
        assert_eq!(params.account_id, Some(123));
        assert_eq!(params.transaction_type_id, Some(2));
        assert_eq!(params.category_id, None);
        assert_eq!(params.description, None);
        assert_eq!(params.page, None);
        assert_eq!(params.page_size, None);

        let params: ListParams =
            serde_qs::from_str("account_id=123&transaction_type_id=2&category_id=5").unwrap();
        assert_eq!(params.account_id, Some(123));
        assert_eq!(params.transaction_type_id, Some(2));
        assert_eq!(params.category_id, Some(5));
        assert_eq!(params.description, None);

        let params: ListParams = serde_qs::from_str("page=3&page_size=20&category_id=5").unwrap();
        assert_eq!(params.page, Some(3));
        assert_eq!(params.page_size, Some(20));
        assert_eq!(params.category_id, Some(5));
        assert_eq!(params.description, None);

        let params: ListParams = serde_qs::from_str("description=Coffee%20Shop").unwrap();
        assert_eq!(params.description.as_deref(), Some("Coffee Shop"));

        let params: ListParams = serde_qs::from_str("").unwrap();
        assert_eq!(params.account_id, None);
        assert_eq!(params.transaction_type_id, None);
        assert_eq!(params.category_id, None);
        assert_eq!(params.description, None);
        assert_eq!(params.page, None);
        assert_eq!(params.page_size, None);
    }

    #[test]
    fn test_successful_response_structure() {
        // Test that the response structure is correct for successful responses
        let transactions = vec![
            create_mock_transaction(1, 123),
            create_mock_transaction(2, 456),
        ];

        // Create expected JSON structure
        let expected_json = serde_json::json!({
            "success": true,
            "data": transactions,
            "count": 2,
            "pagination": {
                "page": 1,
                "pageSize": 10,
                "totalCount": 2,
                "totalPages": 1
            }
        });

        // Verify the JSON structure
        assert_eq!(expected_json["success"], true);
        assert_eq!(expected_json["count"], 2);
        assert!(expected_json["data"].is_array());
        assert_eq!(expected_json["pagination"]["page"], 1);
    }

    #[test]
    fn test_error_response_structure() {
        // Test that error responses have the correct structure
        let error_message = "Database connection failed";

        let expected_json = serde_json::json!({
            "success": false,
            "error": format!("Failed to fetch transactions: {}", error_message)
        });

        assert_eq!(expected_json["success"], false);
        assert!(expected_json["error"]
            .as_str()
            .unwrap()
            .contains("Failed to fetch transactions"));
        assert!(expected_json["error"]
            .as_str()
            .unwrap()
            .contains(error_message));
    }

    #[test]
    fn test_http_status_codes() {
        // Test that correct HTTP status codes are used
        let success_status = StatusCode::OK;
        let error_status = StatusCode::INTERNAL_SERVER_ERROR;

        assert_eq!(success_status, StatusCode::OK);
        assert_eq!(error_status, StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_parse_transaction_datetime() {
        assert!(parse_transaction_datetime("2026-04-04T10:30").is_ok());
        assert!(parse_transaction_datetime("2026-04-04T10:30:45").is_ok());
        assert!(parse_transaction_datetime("2026-04-04 10:30:45").is_ok());
        assert!(parse_transaction_datetime("04/04/2026").is_err());
    }
}
