use axum::{extract::State, http::StatusCode, response::IntoResponse};
use sqlx::MySqlPool;

use crate::services::category_service;

pub async fn list_categories(State(pool): State<MySqlPool>) -> impl IntoResponse {
    match category_service::list_categories(&pool).await {
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

#[cfg(test)]
mod tests {
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
}
