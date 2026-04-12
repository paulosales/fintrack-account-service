use crate::app_state::AppState;
use axum::{routing::get, Router};

use crate::controllers::transaction_type_controller;

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/transaction-types",
        get(transaction_type_controller::list_transaction_types),
    )
}
