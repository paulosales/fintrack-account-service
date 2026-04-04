use crate::controllers::transaction_controller;
use axum::{routing::get, Router};

pub fn routes() -> Router<sqlx::MySqlPool> {
    Router::new().route(
        "/transactions",
        get(transaction_controller::list_transactions),
    )
}
