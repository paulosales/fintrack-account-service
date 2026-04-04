use axum::{routing::get, Router};

use crate::controllers::transaction_category_total_controller;

pub fn routes() -> Router<sqlx::MySqlPool> {
    Router::new()
        .route(
            "/transaction-category-totals",
            get(transaction_category_total_controller::list_transaction_category_totals),
        )
        .route(
            "/transaction-category-totals/details",
            get(transaction_category_total_controller::list_transaction_category_total_details),
        )
}
