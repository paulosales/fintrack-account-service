use crate::models::pagination::{build_pagination_meta, normalize_page, normalize_page_size};
use crate::services::budget_setup_service;
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::MySqlPool;

#[derive(Debug, Deserialize)]
pub struct BudgetSetupListParams {
    page: Option<u32>,
    page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetSetupPayload {
    account_id: i64,
    date: String,
    is_repeatle: bool,
    repeat_frequency: Option<String>,
    end_date: Option<String>,
    description: String,
    amount: f64,
    note: Option<String>,
}

fn parse_budget_setup_date(value: &str) -> Result<NaiveDate, &'static str> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| "Invalid date format")
}

fn map_budget_setup_payload(
    payload: BudgetSetupPayload,
) -> Result<budget_setup_service::BudgetSetupMutationPayload, &'static str> {
    let end_date = match payload.end_date {
        Some(value) if !value.is_empty() => Some(parse_budget_setup_date(&value)?),
        _ => None,
    };

    Ok(budget_setup_service::BudgetSetupMutationPayload {
        account_id: payload.account_id,
        date: parse_budget_setup_date(&payload.date)?,
        is_repeatle: payload.is_repeatle,
        repeat_frequency: payload.repeat_frequency,
        end_date,
        description: payload.description,
        amount: payload.amount,
        note: payload.note,
    })
}

pub async fn list_budget_setups(
    State(pool): State<MySqlPool>,
    Query(params): Query<BudgetSetupListParams>,
) -> impl IntoResponse {
    let page = normalize_page(params.page);
    let page_size = normalize_page_size(params.page_size);

    match budget_setup_service::list_budget_setups(&pool, page, page_size).await {
        Ok((setups, total_count)) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": setups,
                "count": setups.len(),
                "pagination": build_pagination_meta(page, page_size, total_count)
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch budget setups: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn create_budget_setup(
    State(pool): State<MySqlPool>,
    Json(payload): Json<BudgetSetupPayload>,
) -> impl IntoResponse {
    let payload = match map_budget_setup_payload(payload) {
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

    match budget_setup_service::create_budget_setup(&pool, payload).await {
        Ok(setup) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({
                "success": true,
                "data": setup
            })),
        )
            .into_response(),
        Err(error)
            if error.to_string().contains("required")
                || error.to_string().contains("Invalid")
                || error.to_string().contains("cannot be before") =>
        {
            (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": error.to_string()
                })),
            )
                .into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to create budget setup: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn update_budget_setup(
    State(pool): State<MySqlPool>,
    Path(budget_setup_id): Path<i64>,
    Json(payload): Json<BudgetSetupPayload>,
) -> impl IntoResponse {
    let payload = match map_budget_setup_payload(payload) {
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

    match budget_setup_service::update_budget_setup(&pool, budget_setup_id, payload).await {
        Ok(setup) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": setup
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
        Err(error)
            if error.to_string().contains("required")
                || error.to_string().contains("Invalid")
                || error.to_string().contains("cannot be before") =>
        {
            (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({
                    "success": false,
                    "error": error.to_string()
                })),
            )
                .into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to update budget setup: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn delete_budget_setup(
    State(pool): State<MySqlPool>,
    Path(budget_setup_id): Path<i64>,
) -> impl IntoResponse {
    match budget_setup_service::delete_budget_setup(&pool, budget_setup_id).await {
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
        Err(error) if error.to_string().to_lowercase().contains("foreign key") => (
            StatusCode::CONFLICT,
            axum::Json(serde_json::json!({
                "success": false,
                "error": "Budget setup cannot be deleted because it already has generated budgets"
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to delete budget setup: {}", error)
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_setup_list_params_deserialization() {
        let params: BudgetSetupListParams = serde_qs::from_str("page=2&page_size=20").unwrap();

        assert_eq!(params.page, Some(2));
        assert_eq!(params.page_size, Some(20));
    }

    #[test]
    fn test_map_budget_setup_payload() {
        let payload = map_budget_setup_payload(BudgetSetupPayload {
            account_id: 1,
            date: "2026-04-16".to_string(),
            is_repeatle: true,
            repeat_frequency: Some("MONTHLY".to_string()),
            end_date: Some("2026-12-16".to_string()),
            description: "Insurance".to_string(),
            amount: -120.0,
            note: Some("Monthly".to_string()),
        })
        .unwrap();

        assert_eq!(payload.account_id, 1);
        assert!(payload.is_repeatle);
        assert_eq!(payload.repeat_frequency.as_deref(), Some("MONTHLY"));
        assert_eq!(
            payload.end_date,
            Some(NaiveDate::from_ymd_opt(2026, 12, 16).unwrap())
        );
    }
}
