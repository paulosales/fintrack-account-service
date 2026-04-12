use anyhow::Context;
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::app_state::AppState;

/// JWT claims extracted from a Keycloak RS256 access token.
///
/// Attached to the request extensions by this middleware so downstream
/// handlers can read the caller's identity without an additional network hop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub email: Option<String>,
    /// Full name claim (standard OIDC).
    pub name: Option<String>,
    /// Username claim — Keycloak fallback when `name` is not mapped.
    pub preferred_username: Option<String>,
}

// ── JWKS helpers ─────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RsaJwk {
    kid: Option<String>,
    kty: String,
    n: String,
    e: String,
}

#[derive(Deserialize)]
struct JwksResponse {
    keys: Vec<RsaJwk>,
}

/// Fetch the Keycloak JWKS certs endpoint, parse RSA keys, and repopulate the
/// in-memory cache.  Called lazily the first time a token is validated and
/// whenever an unknown `kid` is encountered (key rotation).
async fn refresh_jwks(state: &AppState) -> anyhow::Result<()> {
    let certs_url = format!("{}/protocol/openid-connect/certs", state.keycloak_realm_url);

    let jwks: JwksResponse = state
        .http_client
        .get(&certs_url)
        .send()
        .await
        .context("JWKS request failed")?
        .error_for_status()
        .context("JWKS endpoint returned an error status")?
        .json()
        .await
        .context("Failed to parse JWKS JSON")?;

    let mut cache = state.jwks_cache.write().await;
    for key in jwks.keys {
        if key.kty != "RSA" {
            continue;
        }
        if let Some(kid) = key.kid {
            match DecodingKey::from_rsa_components(&key.n, &key.e) {
                Ok(decoding_key) => {
                    cache.insert(kid, decoding_key);
                }
                Err(e) => {
                    eprintln!("Warning: skipping JWKS key '{kid}': {e}");
                }
            }
        }
    }
    Ok(())
}

/// Attempt to validate `token` using the entry for `kid` in `cache`.
///
/// Returns `None` when the key is absent (cache miss — trigger a refresh).
fn try_validate(
    token: &str,
    kid: &str,
    cache: &HashMap<String, DecodingKey>,
) -> Option<Result<JwtClaims, jsonwebtoken::errors::Error>> {
    let key = cache.get(kid)?;
    // Validate RS256 signature and token expiry.
    // Issuer / audience validation is intentionally skipped here; configure it
    // via environment-specific settings in production.
    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_aud = false;
    Some(decode::<JwtClaims>(token, key, &validation).map(|d| d.claims))
}

// ── Middleware ────────────────────────────────────────────────────────────────

/// Tower middleware that validates a Bearer JWT using Keycloak's public JWKS.
///
/// On success, the decoded [`JwtClaims`] are attached to the request
/// extensions so handlers can read the caller's identity with
/// `req.extensions().get::<JwtClaims>()` — no additional network call needed.
///
/// Returns `401 Unauthorized` when:
/// - The `Authorization: Bearer <token>` header is absent or malformed.
/// - The JWT header is missing a `kid` field.
/// - Token signature or expiry validation fails.
/// - The JWKS endpoint is unreachable and the key is not already cached.
pub async fn validate_bearer_token(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_owned();

    let header = decode_header(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let kid = header.kid.ok_or(StatusCode::UNAUTHORIZED)?;

    // 1. Try the in-memory cache first (fast path, no network).
    let claims = {
        let cache = state.jwks_cache.read().await;
        try_validate(&token, &kid, &cache)
    };

    let claims = match claims {
        Some(Ok(c)) => c,
        _ => {
            // Cache miss or validation failure — refresh JWKS once and retry.
            if refresh_jwks(&state).await.is_err() {
                return Err(StatusCode::UNAUTHORIZED);
            }
            let cache = state.jwks_cache.read().await;
            match try_validate(&token, &kid, &cache) {
                Some(Ok(c)) => c,
                _ => return Err(StatusCode::UNAUTHORIZED),
            }
        }
    };

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}
