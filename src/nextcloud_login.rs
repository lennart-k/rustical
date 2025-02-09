use actix_web::{
    http::header::{self, ContentType},
    web::{self, Data, Form, Json, Path, ServiceConfig},
    HttpRequest, HttpResponse, Responder,
};
use dashmap::DashMap;
use rand::{distributions::Alphanumeric, Rng};
use rustical_store::auth::{AuthenticationMiddleware, AuthenticationProvider, User};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct NextcloudFlows {
    tokens: DashMap<String, String>,
    completed_flows: DashMap<String, NextcloudSuccessResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NextcloudLoginPoll {
    token: String,
    endpoint: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NextcloudLoginResponse {
    poll: NextcloudLoginPoll,
    login: String,
}

async fn post_nextcloud_login(
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
    state.tokens.insert(flow_id, token.to_owned());
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
struct NextcloudSuccessResponse {
    server: String,
    login_name: String,
    app_password: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NextcloudPollForm {
    token: String,
}

async fn post_nextcloud_poll<AP: AuthenticationProvider>(
    state: Data<NextcloudFlows>,
    form: Form<NextcloudPollForm>,
    path: Path<String>,
    auth_provider: Data<AP>,
    req: HttpRequest,
) -> Result<HttpResponse, rustical_store::Error> {
    let flow = path.into_inner();
    match state.tokens.get(&flow) {
        None => return Ok(HttpResponse::Unauthorized().finish()),
        Some(dash_ref) if &form.token != dash_ref.value() => {
            return Ok(HttpResponse::Unauthorized().finish())
        }
        _ => {}
    };

    let app_name = req
        .headers()
        .get(header::USER_AGENT)
        .map(|val| val.to_str().unwrap_or("Client"))
        .unwrap_or("Client");

    if let Some((_, response)) = state.completed_flows.remove(&flow) {
        auth_provider
            .add_app_token(
                &response.login_name,
                app_name.to_owned(),
                response.app_password.to_owned(),
            )
            .await?;
        state.tokens.remove(&flow);
        Ok(Json(response).respond_to(&req).map_into_boxed_body())
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}

fn generate_app_token() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .map(char::from)
        .take(64)
        .collect()
}

async fn get_nextcloud_flow(
    user: User,
    state: Data<NextcloudFlows>,
    path: Path<String>,
    req: HttpRequest,
) -> Result<impl Responder, rustical_store::Error> {
    let flow = path.into_inner();
    if !state.tokens.contains_key(&flow) {
        return Ok(HttpResponse::NotFound().body("Login flow not found"));
    }

    state.completed_flows.insert(
        flow,
        NextcloudSuccessResponse {
            server: req.full_url().origin().unicode_serialization(),
            login_name: user.id.to_owned(),
            app_password: generate_app_token(),
        },
    );
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            "<!Doctype html><html><body><h1>Hello {}!</h1><p>Login completed, you may close this page.</p></body></html>",
            user.displayname.unwrap_or(user.id)
        )))
}

pub fn configure_nextcloud_login<AP: AuthenticationProvider>(
    cfg: &mut ServiceConfig,
    nextcloud_flows_state: Arc<NextcloudFlows>,
    auth_provider: Arc<AP>,
) {
    cfg.service(
        web::scope("")
            .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
            .app_data(Data::from(nextcloud_flows_state))
            .app_data(Data::from(auth_provider.clone()))
            .service(web::resource("/index.php/login/v2").post(post_nextcloud_login))
            .service(
                web::resource("/login/v2/poll/{flow}")
                    .name("nc_login_poll")
                    .post(post_nextcloud_poll::<AP>),
            )
            .service(
                web::resource("/login/v2/flow/{flow}")
                    .name("nc_login_flow")
                    .get(get_nextcloud_flow),
            ),
    );
}
