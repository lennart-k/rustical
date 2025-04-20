use crate::{FrontendConfig, OidcConfig, oidc::OidcProviderData};
use actix_session::Session;
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    error::{ErrorNotFound, ErrorUnauthorized},
    web::{Data, Form, Query, Redirect},
};
use askama::Template;
use askama_web::WebTemplate;
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

#[derive(Debug, Deserialize)]
pub struct GetLoginQuery {
    redirect_uri: Option<String>,
}

#[instrument(skip(req, config, oidc_config))]
pub async fn route_get_login(
    Query(GetLoginQuery { redirect_uri }): Query<GetLoginQuery>,
    req: HttpRequest,
    config: Data<FrontendConfig>,
    oidc_config: Data<Option<OidcConfig>>,
) -> impl Responder {
    let oidc_data = oidc_config
        .as_ref()
        .as_ref()
        .map(|oidc_config| OidcProviderData {
            name: &oidc_config.name,
            redirect_url: req
                .url_for_static("frontend_login_oidc")
                .unwrap()
                .to_string(),
        });
    LoginPage {
        redirect_uri,
        allow_password_login: config.allow_password_login,
        oidc_data,
    }
    .respond_to(&req)
}

#[derive(Deserialize)]
pub struct PostLoginForm {
    username: String,
    password: String,
    redirect_uri: Option<String>,
}

#[instrument(skip(req, password, auth_provider, session, config))]
pub async fn route_post_login<AP: AuthenticationProvider>(
    req: HttpRequest,
    Form(PostLoginForm {
        username,
        password,
        redirect_uri,
    }): Form<PostLoginForm>,
    session: Session,
    auth_provider: Data<AP>,
    config: Data<FrontendConfig>,
) -> HttpResponse {
    if !config.allow_password_login {
        return ErrorNotFound("Password authentication disabled").error_response();
    }
    // Ensure that redirect_uri never goes cross-origin
    let default_redirect = "/frontend/user".to_string();
    let redirect_uri = redirect_uri.unwrap_or(default_redirect.clone());
    let redirect_uri = req
        .full_url()
        .join(&redirect_uri)
        .ok()
        .and_then(|uri| req.full_url().make_relative(&uri))
        .unwrap_or(default_redirect);

    if let Ok(Some(user)) = auth_provider.validate_password(&username, &password).await {
        session.insert("user", user.id).unwrap();
        Redirect::to(redirect_uri)
            .see_other()
            .respond_to(&req)
            .map_into_boxed_body()
    } else {
        ErrorUnauthorized("Unauthorized").error_response()
    }
}

pub async fn route_post_logout(req: HttpRequest, session: Session) -> Redirect {
    session.remove("user");
    Redirect::to(req.url_for_static("frontend_login").unwrap().to_string()).see_other()
}
