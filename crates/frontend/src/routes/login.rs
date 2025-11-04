use std::sync::Arc;

use crate::{FrontendConfig, OidcConfig, pages::DefaultLayoutData};
use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension, Form,
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Host;
use http::StatusCode;
use rustical_store::auth::AuthenticationProvider;
use serde::Deserialize;
use tower_sessions::Session;
use tracing::{instrument, warn};
use url::Url;

#[derive(Template, WebTemplate)]
#[template(path = "pages/login.html")]
struct LoginPage<'a> {
    redirect_uri: Option<String>,
    oidc_data: Option<OidcProviderData<'a>>,
    allow_password_login: bool,
}

impl DefaultLayoutData for LoginPage<'_> {
    fn get_user(&self) -> Option<&rustical_store::auth::Principal> {
        None
    }
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
    let oidc_data = oidc_config
        .as_ref()
        .as_ref()
        .map(|oidc_config| OidcProviderData {
            name: &oidc_config.name,
            redirect_url: "/frontend/login/oidc".to_owned(),
        });
    LoginPage {
        redirect_uri,
        allow_password_login: config.allow_password_login,
        oidc_data,
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
    session: Session,
    Host(host): Host,
    Form(PostLoginForm {
        username,
        password,
        redirect_uri,
    }): Form<PostLoginForm>,
) -> Response {
    if !config.allow_password_login {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let default_redirect = "/frontend/user".to_string();
    // Ensure that redirect_uri never goes cross-origin
    let base_url: Url = format!("https://{host}").parse().unwrap();
    let redirect_uri = if let Some(redirect_uri) = redirect_uri {
        if let Ok(redirect_url) = base_url.join(&redirect_uri) {
            if redirect_url.origin() == base_url.origin() {
                redirect_url.path().to_owned()
            } else {
                default_redirect
            }
        } else {
            default_redirect
        }
    } else {
        default_redirect
    };

    if let Ok(Some(user)) = auth_provider.validate_password(&username, &password).await {
        session.insert("user", user.id).await.unwrap();
        Redirect::to(&redirect_uri).into_response()
    } else {
        warn!("Failed password login attempt as {username}");
        StatusCode::UNAUTHORIZED.into_response()
    }
}

pub async fn route_post_logout(session: Session) -> Redirect {
    session.remove_value("user").await.unwrap();
    Redirect::to("/")
}
