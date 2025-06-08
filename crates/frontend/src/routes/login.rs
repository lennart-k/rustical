use std::sync::Arc;

use crate::{FrontendConfig, OidcConfig};
use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension, Form,
    extract::{OriginalUri, Query},
    response::{IntoResponse, Redirect, Response},
};
use http::StatusCode;
use rustical_store::auth::AuthenticationProvider;
use serde::Deserialize;
use tracing::instrument;

#[derive(Template, WebTemplate)]
#[template(path = "pages/login.html")]
struct LoginPage<'a> {
    redirect_uri: Option<String>,
    oidc_data: Option<OidcProviderData<'a>>,
    allow_password_login: bool,
}

struct OidcProviderData<'a> {
    pub name: &'a str,
    pub redirect_url: String,
}

#[derive(Debug, Deserialize)]
pub struct GetLoginQuery {
    redirect_uri: Option<String>,
}

#[instrument(skip(config, oidc_config))]
pub async fn route_get_login(
    Query(GetLoginQuery { redirect_uri }): Query<GetLoginQuery>,
    Extension(config): Extension<FrontendConfig>,
    Extension(oidc_config): Extension<Option<OidcConfig>>,
) -> Response {
    // let oidc_data = oidc_config
    //     .as_ref()
    //     .as_ref()
    //     .map(|oidc_config| OidcProviderData {
    //         name: &oidc_config.name,
    //         redirect_url: req
    //             .url_for_static(ROUTE_NAME_OIDC_LOGIN)
    //             .unwrap()
    //             .to_string(),
    //     });
    LoginPage {
        redirect_uri,
        allow_password_login: config.allow_password_login,
        oidc_data: None,
    }
    .into_response()
}

#[derive(Deserialize)]
pub struct PostLoginForm {
    username: String,
    password: String,
    redirect_uri: Option<String>,
}

// #[instrument(skip(password, auth_provider, config))]
pub async fn route_post_login<AP: AuthenticationProvider>(
    Extension(auth_provider): Extension<Arc<AP>>,
    Extension(config): Extension<FrontendConfig>,
    OriginalUri(orig_uri): OriginalUri,
    Form(PostLoginForm {
        username,
        password,
        redirect_uri,
    }): Form<PostLoginForm>,
) -> Response {
    if !config.allow_password_login {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    // Ensure that redirect_uri never goes cross-origin
    let default_redirect = "/frontend/user".to_string();
    let redirect_uri = redirect_uri.unwrap_or(default_redirect.clone());
    // let redirect_uri = orig_uri
    //     .join(&redirect_uri)
    //     .ok()
    //     .and_then(|uri| orig_uri.make_relative(&uri))
    //     .unwrap_or(default_redirect);

    if let Ok(Some(user)) = auth_provider.validate_password(&username, &password).await {
        // session.insert("user", user.id).unwrap();
        Redirect::to(&redirect_uri).into_response()
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

pub async fn route_post_logout() -> Redirect {
    // session.remove("user");
    Redirect::to("/")
}
