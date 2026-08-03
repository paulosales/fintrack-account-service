use crate::app_state::AppState;
use axum::{
    routing::{get, put},
    Router,
};

use crate::controllers::transaction_type_controller;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/transaction-types",
            get(transaction_type_controller::list_transaction_types)
                .post(transaction_type_controller::create_transaction_type),
        )
        .route(
            "/transaction-types/{id}",
            put(transaction_type_controller::update_transaction_type)
                .delete(transaction_type_controller::delete_transaction_type),
        )
}
