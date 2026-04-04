use crate::controllers::transaction_controller;
use axum::{
    routing::{get, put},
    Router,
};

pub fn routes() -> Router<sqlx::MySqlPool> {
    Router::new()
        .route(
            "/transactions",
            get(transaction_controller::list_transactions)
                .post(transaction_controller::create_transaction),
        )
        .route(
            "/transactions/{id}",
            put(transaction_controller::update_transaction)
                .delete(transaction_controller::delete_transaction),
        )
}
