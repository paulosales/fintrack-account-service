use crate::app_state::AppState;
use axum::{routing::get, Router};

use crate::controllers::budget_setup_controller;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/budget-setups",
            get(budget_setup_controller::list_budget_setups)
                .post(budget_setup_controller::create_budget_setup),
        )
        .route(
            "/budget-setups/{id}",
            axum::routing::put(budget_setup_controller::update_budget_setup)
                .delete(budget_setup_controller::delete_budget_setup),
        )
}
