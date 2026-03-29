use axum::Router;
use tokio::net::TcpListener;
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
        .with_state(pool);

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind");
    
    println!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app)
        .await
        .expect("Server error");
}