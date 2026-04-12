use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
mod app_state;
mod cache;
mod controllers;
mod db;
mod models;
mod routes;
mod services;

use app_state::AppState;

#[tokio::main]
async fn main() {
    let pool = db::get_pool().await;

    db::run_migrations(&pool).await;

    let cache = cache::create_connection_manager().await;

    println!("Database migrations applied successfully");
    println!("Redis cache connected");

    let state = AppState { pool, cache };

    let app = Router::new()
        .merge(routes::transaction_routes::routes())
        .merge(routes::budget_setup_routes::routes())
        .merge(routes::budget_routes::routes())
        .merge(routes::transaction_category_total_routes::routes())
        .merge(routes::account_routes::routes())
        .merge(routes::transaction_type_routes::routes())
        .merge(routes::category_routes::routes())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind");

    println!("Server running on http://0.0.0.0:3001");

    axum::serve(listener, app).await.expect("Server error");
}
