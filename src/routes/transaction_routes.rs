use crate::app_state::AppState;
use crate::controllers::transaction_controller;
use axum::{
    routing::{get, put},
    Router,
};

pub fn routes() -> Router<AppState> {
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
        .route(
            "/transactions/{id}/sub_transactions",
            get(transaction_controller::get_sub_transactions)
                .post(transaction_controller::create_sub_transaction),
        )
        .route(
            "/transactions/{transaction_id}/sub_transactions/{id}",
            put(transaction_controller::update_sub_transaction)
                .delete(transaction_controller::delete_sub_transaction),
        )
}
