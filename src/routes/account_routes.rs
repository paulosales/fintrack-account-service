use crate::app_state::AppState;
use crate::controllers::account_controller;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/accounts", get(account_controller::list_accounts))
        .route("/accounts", post(account_controller::create_account))
        .route("/accounts/{id}", put(account_controller::update_account))
        .route("/accounts/{id}", delete(account_controller::delete_account))
}
