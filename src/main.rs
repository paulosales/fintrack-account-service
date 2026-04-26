use axum::middleware as mw;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
mod app_state;
mod cache;
mod controllers;
mod db;
mod middleware;
mod models;
mod rabbitmq;
mod routes;
mod services;

use app_state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("account_service=info".parse().unwrap()),
        )
        .init();

    let pool = db::get_pool().await;

    db::run_migrations(&pool).await;

    let cache = cache::create_connection_manager().await;

    println!("Database migrations applied successfully");
    println!("Redis cache connected");

    let keycloak_realm_url = std::env::var("KEYCLOAK_REALM_URL")
        .unwrap_or_else(|_| "http://keycloak:8080/realms/fintrack".to_string());

    let settings_service_url = std::env::var("SETTINGS_SERVICE_URL")
        .unwrap_or_else(|_| "http://settings-service:3004".to_string());

    let currency_service_url = std::env::var("CURRENCY_SERVICE_URL")
        .unwrap_or_else(|_| "http://currency-service:3003".to_string());

    // Clone pool for the RabbitMQ consumer before it is moved into AppState
    let rabbitmq_pool = pool.clone();

    let state = AppState {
        pool,
        cache,
        keycloak_realm_url,
        http_client: reqwest::Client::new(),
        jwks_cache: app_state::new_jwks_cache(),
        settings_service_url,
        currency_service_url,
    };

    let app = Router::new()
        .merge(routes::transaction_routes::routes())
        .merge(routes::budget_setup_routes::routes())
        .merge(routes::budget_routes::routes())
        .merge(routes::transaction_category_total_routes::routes())
        .merge(routes::account_routes::routes())
        .merge(routes::account_type_routes::routes())
        .merge(routes::transaction_type_routes::routes())
        .merge(routes::category_routes::routes())
        .route_layer(mw::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware::validate_bearer_token,
        ))
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

    // Spawn the RabbitMQ consumer as a background task
    tokio::spawn(async move {
        rabbitmq::consumer::start_consumer(rabbitmq_pool).await;
    });

    axum::serve(listener, app).await.expect("Server error");
}
