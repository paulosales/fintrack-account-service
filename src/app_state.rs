use axum::extract::FromRef;
use jsonwebtoken::DecodingKey;
use redis::aio::ConnectionManager;
use sqlx::MySqlPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory JWKS cache keyed by the JWT `kid` (key ID) header field.
/// Populated lazily from Keycloak's JWKS endpoint; refreshed automatically
/// when an unknown `kid` is encountered.
pub type JwksCache = Arc<RwLock<HashMap<String, DecodingKey>>>;

pub fn new_jwks_cache() -> JwksCache {
    Arc::new(RwLock::new(HashMap::new()))
}

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub cache: ConnectionManager,
    /// Base Keycloak realm URL, e.g. `http://keycloak:8080/realms/fintrack`.
    /// JWKS endpoint: `{keycloak_realm_url}/protocol/openid-connect/certs`.
    pub keycloak_realm_url: String,
    pub http_client: reqwest::Client,
    pub jwks_cache: JwksCache,
    pub settings_service_url: String,
    pub currency_service_url: String,
}

impl FromRef<AppState> for MySqlPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for ConnectionManager {
    fn from_ref(state: &AppState) -> Self {
        state.cache.clone()
    }
}
