use axum::{Router, routing::get};
use crate::controllers::transaction_controller;

pub fn routes() -> Router<sqlx::MySqlPool> {
    Router::new()
        .route("/transactions", get(transaction_controller::list_transactions))
}