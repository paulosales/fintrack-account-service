use axum::{Router, routing::get};
use crate::controllers::account_controller;

pub fn routes() -> Router<sqlx::MySqlPool> {
    Router::new()
        .route("/accounts", get(account_controller::list_accounts))
}
