use crate::models::pagination::{build_pagination_meta, normalize_page, normalize_page_size};
use crate::services::transaction_category_total_service;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Debug, Deserialize)]
pub struct TransactionCategoryTotalsParams {
    month: Option<u32>,
    year: Option<i32>,
    category_id: Option<i64>,
    page: Option<u32>,
    page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionCategoryTotalDetailsParams {
    month: u32,
    year: i32,
    category_id: i64,
}

pub async fn list_transaction_category_totals(
    State(pool): State<MySqlPool>,
    Query(params): Query<TransactionCategoryTotalsParams>,
) -> impl IntoResponse {
    let page = normalize_page(params.page);
    let page_size = normalize_page_size(params.page_size);

    match transaction_category_total_service::list_transaction_category_totals(
        &pool,
        params.month,
        params.year,
        params.category_id,
        page,
        page_size,
    )
    .await
    {
        Ok((totals, total_count)) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": totals,
                "count": totals.len(),
                "pagination": build_pagination_meta(page, page_size, total_count)
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch transaction category totals: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn list_transaction_category_total_details(
    State(pool): State<MySqlPool>,
    Query(params): Query<TransactionCategoryTotalDetailsParams>,
) -> impl IntoResponse {
    match transaction_category_total_service::list_transaction_category_total_details(
        &pool,
        params.month,
        params.year,
        params.category_id,
    )
    .await
    {
        Ok(details) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": details,
                "count": details.len()
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch transaction category total details: {}", error)
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totals_params_deserialization() {
        let params: TransactionCategoryTotalsParams =
            serde_qs::from_str("month=4&year=2026&category_id=7").unwrap();

        assert_eq!(params.month, Some(4));
        assert_eq!(params.year, Some(2026));
        assert_eq!(params.category_id, Some(7));
        assert_eq!(params.page, None);
        assert_eq!(params.page_size, None);

        let params: TransactionCategoryTotalsParams =
            serde_qs::from_str("month=4&year=2026&category_id=7&page=2&page_size=30").unwrap();
        assert_eq!(params.page, Some(2));
        assert_eq!(params.page_size, Some(30));
    }

    #[test]
    fn test_details_params_deserialization() {
        let params: TransactionCategoryTotalDetailsParams =
            serde_qs::from_str("month=4&year=2026&category_id=7").unwrap();

        assert_eq!(params.month, 4);
        assert_eq!(params.year, 2026);
        assert_eq!(params.category_id, 7);
    }
}
