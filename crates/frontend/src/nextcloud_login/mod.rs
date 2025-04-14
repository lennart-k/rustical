use actix_web::{
    http::StatusCode,
    middleware::ErrorHandlers,
    web::{self, Data, ServiceConfig},
};
use chrono::{DateTime, Utc};
use routes::{get_nextcloud_flow, post_nextcloud_flow, post_nextcloud_login, post_nextcloud_poll};
use rustical_store::auth::{AuthenticationMiddleware, AuthenticationProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
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

use crate::{session_middleware, unauthorized_handler};

pub fn configure_nextcloud_login<AP: AuthenticationProvider>(
    cfg: &mut ServiceConfig,
    nextcloud_flows_state: Arc<NextcloudFlows>,
    auth_provider: Arc<AP>,
    frontend_secret: [u8; 64],
) {
    cfg.service(
        web::scope("/index.php/login/v2")
            .wrap(ErrorHandlers::new().handler(StatusCode::UNAUTHORIZED, unauthorized_handler))
            .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
            .wrap(session_middleware(frontend_secret))
            .app_data(Data::from(nextcloud_flows_state))
            .app_data(Data::from(auth_provider.clone()))
            .service(web::resource("").post(post_nextcloud_login))
            .service(
                web::resource("/poll/{flow}")
                    .name("nc_login_poll")
                    .post(post_nextcloud_poll::<AP>),
            )
            .service(
                web::resource("/flow/{flow}")
                    .name("nc_login_flow")
                    .get(get_nextcloud_flow)
                    .post(post_nextcloud_flow),
            ),
    );
}
