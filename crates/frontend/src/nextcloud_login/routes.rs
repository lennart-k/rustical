use super::{
    NextcloudFlow, NextcloudFlows, NextcloudLoginPoll, NextcloudLoginResponse,
    NextcloudSuccessResponse,
};
use crate::routes::app_token::generate_app_token;
use askama::Template;
use axum::{
    Extension, Form, Json,
    extract::Path,
    response::{Html, IntoResponse, Response},
};
use axum_extra::{TypedHeader, extract::Host};
use chrono::{Duration, Utc};
use headers::UserAgent;
use http::StatusCode;
use rustical_store::auth::{AuthenticationProvider, Principal};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::instrument;

pub async fn post_nextcloud_login(
    Extension(state): Extension<Arc<NextcloudFlows>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Host(host): Host,
) -> Json<NextcloudLoginResponse> {
    let flow_id = uuid::Uuid::new_v4().to_string();
    let token = uuid::Uuid::new_v4().to_string();

    let app_name = user_agent.to_string();
    let mut flows = state.flows.write().await;
    // Flows must not last longer than 10 minutes
    // We also enforce that condition here to prevent a memory leak where unpolled flows would
    // never be cleaned up
    flows.retain(|_, flow| Utc::now() - flow.created_at < Duration::minutes(10));
    flows.insert(
        flow_id.clone(),
        NextcloudFlow {
            app_name: app_name.clone(),
            created_at: Utc::now(),
            token: token.clone(),
            response: None,
        },
    );
    Json(NextcloudLoginResponse {
        login: format!("https://{host}/index.php/login/v2/flow/{flow_id}"),
        poll: NextcloudLoginPoll {
            token,
            endpoint: format!("https://{host}/index.php/login/v2/poll/{flow_id}"),
        },
    })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextcloudPollForm {
    token: String,
}

pub async fn post_nextcloud_poll<AP: AuthenticationProvider>(
    Extension(state): Extension<Arc<NextcloudFlows>>,
    Path(flow_id): Path<String>,
    Extension(auth_provider): Extension<Arc<AP>>,
    Form(form): Form<NextcloudPollForm>,
) -> Result<Response, rustical_store::Error> {
    let mut flows = state.flows.write().await;

    // Flows must not last longer than 10 minutes
    flows.retain(|_, flow| Utc::now() - flow.created_at < Duration::minutes(10));

    if let Some(flow) = flows.get(&flow_id).cloned() {
        if flow.token != form.token {
            return Ok(StatusCode::UNAUTHORIZED.into_response());
        }
        if let Some(response) = &flow.response {
            auth_provider
                .add_app_token(
                    &response.login_name,
                    flow.app_name.clone(),
                    response.app_password.clone(),
                )
                .await?;
            flows.remove(&flow_id);
            Ok(Json(response).into_response())
        } else {
            // Not done yet, re-insert flow
            Ok(StatusCode::NOT_FOUND.into_response())
        }
    } else {
        Ok(StatusCode::UNAUTHORIZED.into_response())
    }
}

#[derive(Template)]
#[template(path = "pages/nextcloud_login/form.html")]
struct NextcloudLoginPage {
    username: String,
    app_name: String,
}

#[instrument(skip(state))]
pub async fn get_nextcloud_flow(
    Extension(state): Extension<Arc<NextcloudFlows>>,
    Path(flow_id): Path<String>,
    user: Principal,
) -> Result<Response, rustical_store::Error> {
    if let Some(flow) = state.flows.read().await.get(&flow_id) {
        Ok(Html(
            NextcloudLoginPage {
                username: user.displayname.unwrap_or(user.id),
                app_name: flow.app_name.clone(),
            }
            .render()
            .unwrap(),
        )
        .into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, "Login flow not found").into_response())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NextcloudAuthorizeForm {
    app_name: String,
}

#[derive(Template)]
#[template(path = "pages/nextcloud_login/success.html")]
struct NextcloudLoginSuccessPage {
    app_name: String,
}

#[instrument(skip(state))]
pub async fn post_nextcloud_flow(
    user: Principal,
    Extension(state): Extension<Arc<NextcloudFlows>>,
    Path(flow_id): Path<String>,
    Host(host): Host,
    Form(form): Form<NextcloudAuthorizeForm>,
) -> Result<Response, rustical_store::Error> {
    if let Some(flow) = state.flows.write().await.get_mut(&flow_id) {
        flow.app_name = form.app_name;
        flow.response = Some(NextcloudSuccessResponse {
            server: format!("https://{host}"),
            login_name: user.id.clone(),
            app_password: generate_app_token(),
        });
        Ok(Html(
            NextcloudLoginSuccessPage {
                app_name: flow.app_name.clone(),
            }
            .render()
            .unwrap(),
        )
        .into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, "Login flow not found").into_response())
    }
}
