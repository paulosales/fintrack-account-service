use crate::app_state::AppState;
use axum::{routing::get, Router};

use crate::controllers::budget_controller;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/budgets",
            get(budget_controller::list_budget_month_totals).post(budget_controller::create_budget),
        )
        .route(
            "/budgets/details",
            get(budget_controller::list_budget_details),
        )
        .route(
            "/budgets/generate",
            axum::routing::post(budget_controller::generate_budgets),
        )
        .route(
            "/budgets/{id}",
            axum::routing::put(budget_controller::update_budget)
                .delete(budget_controller::delete_budget),
        )
}
