use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use redis::aio::ConnectionManager;
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::services::category_service;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryPayload {
    name: String,
}

fn map_payload(
    payload: CategoryPayload,
) -> Result<crate::models::categories::CategoryUpsert, &'static str> {
    let name = payload.name.trim().to_string();

    if name.is_empty() {
        return Err("Category name is required");
    }

    Ok(crate::models::categories::CategoryUpsert { name })
}

pub async fn list_categories(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
) -> impl IntoResponse {
    match category_service::list_categories(&pool, &mut cache).await {
        Ok(categories) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": categories,
                "count": categories.len()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to fetch categories: {}", e)
            })),
        )
            .into_response(),
    }
}

pub async fn create_category(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Json(payload): Json<CategoryPayload>,
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

    match category_service::create_category(&pool, &mut cache, payload).await {
        Ok(category) => (
            StatusCode::CREATED,
            axum::Json(serde_json::json!({
                "success": true,
                "data": category
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to create category: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn update_category(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Path(category_id): Path<i64>,
    Json(payload): Json<CategoryPayload>,
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

    match category_service::update_category(&pool, &mut cache, category_id, payload).await {
        Ok(category) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "data": category
            })),
        )
            .into_response(),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": format!("Failed to update category: {}", error)
            })),
        )
            .into_response(),
    }
}

pub async fn delete_category(
    State(pool): State<MySqlPool>,
    State(mut cache): State<ConnectionManager>,
    Path(category_id): Path<i64>,
) -> impl IntoResponse {
    match category_service::delete_category(&pool, &mut cache, category_id).await {
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
                "error": format!("Failed to delete category: {}", error)
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::map_payload;
    use crate::models::categories::Category;

    #[test]
    fn test_create_mock_category() {
        let category = Category {
            id: 1,
            name: "Groceries".to_string(),
        };

        assert_eq!(category.id, 1);
        assert_eq!(category.name, "Groceries");
    }

    #[test]
    fn test_map_payload_trims_name() {
        let payload = super::CategoryPayload {
            name: "  Utilities  ".to_string(),
        };

        let mapped = map_payload(payload).unwrap();

        assert_eq!(mapped.name, "Utilities");
    }

    #[test]
    fn test_map_payload_rejects_empty_name() {
        let payload = super::CategoryPayload {
            name: "   ".to_string(),
        };

        assert_eq!(
            map_payload(payload).unwrap_err(),
            "Category name is required"
        );
    }
}
