use crate::app_state::AppState;
use crate::controllers::account_controller;
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/accounts", get(account_controller::list_accounts))
}
