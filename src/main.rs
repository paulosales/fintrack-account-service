use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
mod routes;
mod controllers;
mod services;
mod models;
mod db;

#[tokio::main]
async fn main() {
    let pool = db::get_pool().await;

    let app = Router::new()
        .merge(routes::transaction_routes::routes())
        .merge(routes::account_routes::routes())
        .merge(routes::transaction_type_routes::routes())
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any))
        .with_state(pool);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");
    
    println!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app)
        .await
        .expect("Server error");
}