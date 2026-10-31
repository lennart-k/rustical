#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
use crate::routes::{route_get_oidc_callback, route_post_oidc};
use axum::{
    Extension, Router,
    routing::{get, post},
};
pub use config::OidcConfig;
use openidconnect::{CsrfToken, Nonce, PkceCodeVerifier};
use serde::{Deserialize, Serialize};
pub use user_store::UserStore;

mod config;
mod error;
mod routes;
mod user_store;

const SESSION_KEY_OIDC_STATE: &str = "oidc_state";

#[derive(Debug, Deserialize, Serialize)]
struct OidcState {
    state: CsrfToken,
    nonce: Nonce,
    pkce_verifier: PkceCodeVerifier,
    redirect_uri: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OidcServiceConfig {
    pub default_redirect_path: &'static str,
    pub session_key_user_id: &'static str,
    pub callback_path: &'static str,
}

#[derive(Debug, Deserialize, Serialize)]
struct GroupAdditionalClaims {
    #[serde(default)]
    groups: Option<Vec<String>>,
}

impl openidconnect::AdditionalClaims for GroupAdditionalClaims {}

pub fn oidc_router<US: UserStore>(
    config: OidcConfig,
    service_config: OidcServiceConfig,
    user_store: US,
) -> Router {
    Router::new()
        .route("/", post(route_post_oidc))
        .route("/callback", get(route_get_oidc_callback::<US>))
        .layer(Extension(user_store))
        .layer(Extension(config))
        .layer(Extension(service_config))
}
