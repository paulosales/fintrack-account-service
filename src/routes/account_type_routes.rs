use crate::app_state::AppState;
use crate::controllers::account_type_controller;
use axum::{
    routing::{get, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/account-types",
            get(account_type_controller::list_account_types)
                .post(account_type_controller::create_account_type),
        )
        .route(
            "/account-types/{id}",
            put(account_type_controller::update_account_type)
                .delete(account_type_controller::delete_account_type),
        )
}
