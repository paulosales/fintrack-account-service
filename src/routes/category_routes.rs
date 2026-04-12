use crate::app_state::AppState;
use axum::{
    routing::{get, put},
    Router,
};

use crate::controllers::category_controller;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/categories",
            get(category_controller::list_categories).post(category_controller::create_category),
        )
        .route(
            "/categories/{id}",
            put(category_controller::update_category).delete(category_controller::delete_category),
        )
}
