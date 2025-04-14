use crate::generate_app_token;

use super::{
    NextcloudFlow, NextcloudFlows, NextcloudLoginPoll, NextcloudLoginResponse,
    NextcloudSuccessResponse,
};
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    http::header::{self},
    web::{Data, Form, Html, Json, Path},
};
use askama::Template;
use chrono::{Duration, Utc};
use rustical_store::auth::{AuthenticationProvider, User};
use serde::{Deserialize, Serialize};
use tracing::instrument;

pub(crate) async fn post_nextcloud_login(
    req: HttpRequest,
    state: Data<NextcloudFlows>,
) -> Json<NextcloudLoginResponse> {
    let flow_id = uuid::Uuid::new_v4().to_string();
    let token = uuid::Uuid::new_v4().to_string();
    let poll_url = req
        .resource_map()
        .url_for(&req, "nc_login_poll", [&flow_id])
        .unwrap();
    let flow_url = req
        .resource_map()
        .url_for(&req, "nc_login_flow", [&flow_id])
        .unwrap();

    let app_name = req
        .headers()
        .get(header::USER_AGENT)
        .map(|val| val.to_str().unwrap_or("Unknown client"))
        .unwrap_or("Unknown client");

    let mut flows = state.flows.write().await;
    // Flows must not last longer than 10 minutes
    // We also enforce that condition here to prevent a memory leak where unpolled flows would
    // never be cleaned up
    flows.retain(|_, flow| Utc::now() - flow.created_at < Duration::minutes(10));
    flows.insert(
        flow_id,
        NextcloudFlow {
            app_name: app_name.to_owned(),
            created_at: Utc::now(),
            token: token.to_owned(),
            response: None,
        },
    );
    Json(NextcloudLoginResponse {
        login: flow_url.to_string(),
        poll: NextcloudLoginPoll {
            token,
            endpoint: poll_url.to_string(),
        },
    })
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NextcloudPollForm {
    token: String,
}

pub(crate) async fn post_nextcloud_poll<AP: AuthenticationProvider>(
    state: Data<NextcloudFlows>,
    form: Form<NextcloudPollForm>,
    path: Path<String>,
    auth_provider: Data<AP>,
    req: HttpRequest,
) -> Result<HttpResponse, rustical_store::Error> {
    let flow_id = path.into_inner();
    let mut flows = state.flows.write().await;

    // Flows must not last longer than 10 minutes
    flows.retain(|_, flow| Utc::now() - flow.created_at < Duration::minutes(10));

    if let Some(flow) = flows.get(&flow_id).cloned() {
        if flow.token != form.token {
            return Ok(HttpResponse::Unauthorized().body("Unauthorized"));
        }
        if let Some(response) = &flow.response {
            auth_provider
                .add_app_token(
                    &response.login_name,
                    flow.app_name.to_owned(),
                    response.app_password.to_owned(),
                )
                .await?;
            flows.remove(&flow_id);
            Ok(Json(response).respond_to(&req).map_into_boxed_body())
        } else {
            // Not done yet, re-insert flow
            Ok(HttpResponse::NotFound().finish())
        }
    } else {
        Ok(HttpResponse::Unauthorized().body("Unauthorized"))
    }
}

#[derive(Template)]
#[template(path = "pages/nextcloud_login/form.html")]
struct NextcloudLoginPage {
    username: String,
    app_name: String,
}

#[instrument(skip(state, req))]
pub(crate) async fn get_nextcloud_flow(
    user: User,
    state: Data<NextcloudFlows>,
    path: Path<String>,
    req: HttpRequest,
) -> Result<impl Responder, rustical_store::Error> {
    let flow_id = path.into_inner();
    if let Some(flow) = state.flows.read().await.get(&flow_id) {
        Ok(Html::new(
            NextcloudLoginPage {
                username: user.displayname.unwrap_or(user.id),
                app_name: flow.app_name.to_owned(),
            }
            .render()
            .unwrap(),
        )
        .respond_to(&req)
        .map_into_boxed_body())
    } else {
        Ok(HttpResponse::NotFound().body("Login flow not found"))
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct NextcloudAuthorizeForm {
    app_name: String,
}

#[derive(Template)]
#[template(path = "pages/nextcloud_login/success.html")]
struct NextcloudLoginSuccessPage {
    app_name: String,
}

#[instrument(skip(state, req))]
pub(crate) async fn post_nextcloud_flow(
    user: User,
    state: Data<NextcloudFlows>,
    path: Path<String>,
    req: HttpRequest,
    form: Form<NextcloudAuthorizeForm>,
) -> Result<impl Responder, rustical_store::Error> {
    let flow_id = path.into_inner();
    if let Some(flow) = state.flows.write().await.get_mut(&flow_id) {
        flow.app_name = form.into_inner().app_name;
        flow.response = Some(NextcloudSuccessResponse {
            server: req.full_url().origin().unicode_serialization(),
            login_name: user.id.to_owned(),
            app_password: generate_app_token(),
        });
        Ok(Html::new(
            NextcloudLoginSuccessPage {
                app_name: flow.app_name.to_owned(),
            }
            .render()
            .unwrap(),
        )
        .respond_to(&req)
        .map_into_boxed_body())
    } else {
        Ok(HttpResponse::NotFound().body("Login flow not found"))
    }
}
