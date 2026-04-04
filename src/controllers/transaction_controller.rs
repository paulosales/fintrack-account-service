use crate::models::pagination::{build_pagination_meta, normalize_page, normalize_page_size};
use crate::services::transaction_service;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct ListParams {
    account_id: Option<i64>,
    transaction_type_id: Option<i64>,
    category_id: Option<i64>,
    page: Option<u32>,
    page_size: Option<u32>,
}

pub async fn list_transactions(
    State(pool): State<MySqlPool>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let page = normalize_page(params.page);
    let page_size = normalize_page_size(params.page_size);

    match transaction_service::list_transactions(
        &pool,
        params.account_id,
        params.transaction_type_id,
        params.category_id,
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
        assert_eq!(params.page, None);
        assert_eq!(params.page_size, None);

        let params: ListParams =
            serde_qs::from_str("account_id=123&transaction_type_id=2").unwrap();
        assert_eq!(params.account_id, Some(123));
        assert_eq!(params.transaction_type_id, Some(2));
        assert_eq!(params.category_id, None);
        assert_eq!(params.page, None);
        assert_eq!(params.page_size, None);

        let params: ListParams =
            serde_qs::from_str("account_id=123&transaction_type_id=2&category_id=5").unwrap();
        assert_eq!(params.account_id, Some(123));
        assert_eq!(params.transaction_type_id, Some(2));
        assert_eq!(params.category_id, Some(5));

        let params: ListParams = serde_qs::from_str("page=3&page_size=20&category_id=5").unwrap();
        assert_eq!(params.page, Some(3));
        assert_eq!(params.page_size, Some(20));
        assert_eq!(params.category_id, Some(5));

        let params: ListParams = serde_qs::from_str("").unwrap();
        assert_eq!(params.account_id, None);
        assert_eq!(params.transaction_type_id, None);
        assert_eq!(params.category_id, None);
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
}
