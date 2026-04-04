use axum::{routing::get, Router};

use crate::controllers::category_controller;

pub fn routes() -> Router<sqlx::MySqlPool> {
    Router::new().route("/categories", get(category_controller::list_categories))
}
