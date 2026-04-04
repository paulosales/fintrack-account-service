use crate::controllers::account_controller;
use axum::{routing::get, Router};

pub fn routes() -> Router<sqlx::MySqlPool> {
    Router::new().route("/accounts", get(account_controller::list_accounts))
}
