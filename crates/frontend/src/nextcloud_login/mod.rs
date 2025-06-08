use axum::routing::{get, post};
use axum::{Extension, Router, middleware};
use chrono::{DateTime, Utc};
use routes::{get_nextcloud_flow, post_nextcloud_flow, post_nextcloud_login, post_nextcloud_poll};
use rustical_store::auth::AuthenticationProvider;
use rustical_store::auth::middleware::AuthenticationLayer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer, SessionStore};

use crate::unauthorized_handler;
mod routes;

#[derive(Debug, Clone)]
struct NextcloudFlow {
    app_name: String,
    created_at: DateTime<Utc>,
    token: String,
    response: Option<NextcloudSuccessResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NextcloudSuccessResponse {
    server: String,
    login_name: String,
    app_password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NextcloudLoginResponse {
    poll: NextcloudLoginPoll,
    login: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NextcloudLoginPoll {
    token: String,
    endpoint: String,
}

#[derive(Debug, Default)]
pub struct NextcloudFlows {
    flows: RwLock<HashMap<String, NextcloudFlow>>,
}

pub fn nextcloud_login_router<AP: AuthenticationProvider, S: SessionStore + Clone>(
    nextcloud_flows_state: Arc<NextcloudFlows>,
    auth_provider: Arc<AP>,
    session_store: S,
) -> Router {
    Router::new()
        .route("/poll/{flow}", post(post_nextcloud_poll::<AP>))
        .route(
            "/flow/{flow}",
            get(get_nextcloud_flow).post(post_nextcloud_flow),
        )
        .route("/", post(post_nextcloud_login))
        .layer(Extension(nextcloud_flows_state))
        .layer(Extension(auth_provider.clone()))
        .layer(AuthenticationLayer::new(auth_provider.clone()))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(true)
                .with_same_site(SameSite::Strict)
                .with_expiry(Expiry::OnInactivity(Duration::hours(2))),
        )
        .layer(middleware::from_fn(unauthorized_handler))
}
