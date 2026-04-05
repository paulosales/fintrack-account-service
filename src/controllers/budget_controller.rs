use crate::models::pagination::{build_pagination_meta, normalize_page, normalize_page_size};
use crate::services::budget_service;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Debug, Deserialize)]
pub struct BudgetListParams {
    page: Option<u32>,
    page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BudgetDetailsParams {
    year: i32,
    month: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetPayload {
    account_id: i64,
    date: String,
    amount: f64,
    description: String,
    processed: bool,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetGenerationPayload {
    start_date: String,
    end_date: String,
}

fn parse_budget_date(value: &str) -> Result<NaiveDate, &'static str> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| "Invalid date format")
}

fn map_budget_payload(
    payload: BudgetPayload,
) -> Result<budget_service::BudgetMutationPayload, &'static str> {
    Ok(budget_service::BudgetMutationPayload {
        account_id: payload.account_id,
        date: parse_budget_date(&payload.date)?,
        amount: payload.amount,
        description: payload.description,
        processed: payload.processed,
        note: payload.note,
    })
}

fn map_generation_payload(
    payload: BudgetGenerationPayload,
) -> Result<budget_service::BudgetGenerationPayload, &'static str> {
    Ok(budget_service::BudgetGenerationPayload {
        start_date: parse_budget_date(&payload.start_date)?,
        end_date: parse_budget_date(&payload.end_date)?,
    })
}

pub async fn list_budget_month_totals(
    State(pool): State<MySqlPool>,
    Query(params): Query<BudgetListParams>,
) -> impl IntoResponse {
    let page = normalize_page(params.page);
    let page_size = normalize_page_size(params.page_size);

    match budget_service::list_budget_month_totals(&pool, page, page_size).await {
        Ok((budgets, total_count)) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": budgets,
                "count": budgets.len(),
                "pagination": build_pagination_meta(page, page_size, total_count)
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch budget month totals: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn list_budget_details(
    State(pool): State<MySqlPool>,
    Query(params): Query<BudgetDetailsParams>,
) -> impl IntoResponse {
    match budget_service::list_budget_details(&pool, params.year, params.month).await {
        Ok(budgets) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": budgets,
                "count": budgets.len()
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch budget details: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn create_budget(
    State(pool): State<MySqlPool>,
    Json(payload): Json<BudgetPayload>,
) -> impl IntoResponse {
    let payload = match map_budget_payload(payload) {
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

    match budget_service::create_budget(&pool, payload).await {
        Ok(budget) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({
                "success": true,
                "data": budget
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to create budget: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn update_budget(
    State(pool): State<MySqlPool>,
    Path(budget_id): Path<i64>,
    Json(payload): Json<BudgetPayload>,
) -> impl IntoResponse {
    let payload = match map_budget_payload(payload) {
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

    match budget_service::update_budget(&pool, budget_id, payload).await {
        Ok(budget) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": budget
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
                "error": format!("Failed to update budget: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn delete_budget(
    State(pool): State<MySqlPool>,
    Path(budget_id): Path<i64>,
) -> impl IntoResponse {
    match budget_service::delete_budget(&pool, budget_id).await {
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
                "error": format!("Failed to delete budget: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn generate_budgets(
    State(pool): State<MySqlPool>,
    Json(payload): Json<BudgetGenerationPayload>,
) -> impl IntoResponse {
    let payload = match map_generation_payload(payload) {
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

    match budget_service::generate_budgets(&pool, payload).await {
        Ok(created_count) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "createdCount": created_count
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to generate budgets: {}", error)
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_list_params_deserialization() {
        let params: BudgetListParams = serde_qs::from_str("page=2&page_size=20").unwrap();

        assert_eq!(params.page, Some(2));
        assert_eq!(params.page_size, Some(20));
    }

    #[test]
    fn test_budget_details_params_deserialization() {
        let params: BudgetDetailsParams = serde_qs::from_str("year=2026&month=4").unwrap();

        assert_eq!(params.year, 2026);
        assert_eq!(params.month, 4);
    }

    #[test]
    fn test_parse_budget_date() {
        assert!(parse_budget_date("2026-04-16").is_ok());
        assert!(parse_budget_date("04/16/2026").is_err());
    }

    #[test]
    fn test_generation_payload_mapping() {
        let payload = map_generation_payload(BudgetGenerationPayload {
            start_date: "2026-01-01".to_string(),
            end_date: "2026-12-31".to_string(),
        })
        .unwrap();

        assert_eq!(payload.start_date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(payload.end_date, NaiveDate::from_ymd_opt(2026, 12, 31).unwrap());
    }
}
