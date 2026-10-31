use crate::{FrontendConfig, OidcConfig, pages::DefaultLayoutData};
use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension, Form,
    extract::Query,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::TypedHeader;
use headers::Host;
use http::StatusCode;
use rustical_store::auth::AuthenticationProvider;
use serde::Deserialize;
use std::sync::Arc;
use tower_sessions::Session;
use tracing::{instrument, warn};
use url::Url;

#[derive(Template, WebTemplate)]
#[template(path = "pages/login.html")]
struct LoginPage<'a> {
    redirect_uri: Option<String>,
    oidc_data: Option<OidcProviderData<'a>>,
    allow_password_login: bool,
    /// The page submits its OIDC form by itself (`oidc_auto_redirect`); without
    /// JavaScript the button stays.
    auto_login: bool,
    logged_out: bool,
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
    /// Set by the logout: show the page instead of signing in again right away
    logged_out: Option<String>,
}

/// Whether the login page submits its OIDC form by itself: only when the button
/// is all it would show (password login off, OIDC configured) and not right
/// after a logout, which would otherwise sign the user straight back in for as
/// long as the provider's session lasts.
const fn auto_login(config: &FrontendConfig, oidc_configured: bool, logged_out: bool) -> bool {
    config.oidc_auto_redirect && !config.allow_password_login && oidc_configured && !logged_out
}

#[instrument(skip(config, oidc_config))]
pub async fn route_get_login(
    Query(GetLoginQuery {
        redirect_uri,
        logged_out,
    }): Query<GetLoginQuery>,
    Extension(config): Extension<FrontendConfig>,
    Extension(oidc_config): Extension<Option<OidcConfig>>,
) -> Response {
    let logged_out = logged_out.is_some();
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
        auto_login: auto_login(&config, oidc_data.is_some(), logged_out),
        logged_out,
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
    TypedHeader(host): TypedHeader<Host>,
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
    let redirect_uri = if let Some(redirect_uri) = redirect_uri
        && let Ok(redirect_url) = base_url.join(&redirect_uri)
        && redirect_url.origin() == base_url.origin()
    {
        redirect_url.path().to_owned()
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
    // Not "/": with oidc_auto_redirect that ends on the login page, which would
    // sign the user straight back in
    Redirect::to("/frontend/login?logged_out")
}

#[cfg(test)]
mod tests {
    use super::*;
    use askama::Template;

    const fn config(allow_password_login: bool, oidc_auto_redirect: bool) -> FrontendConfig {
        FrontendConfig {
            enabled: true,
            allow_password_login,
            oidc_auto_redirect,
        }
    }

    #[test]
    fn test_auto_login_only_when_the_button_is_all_there_is() {
        assert!(auto_login(&config(false, true), true, false));
        // a logout shows the page
        assert!(!auto_login(&config(false, true), true, true));
        // the password form must stay reachable
        assert!(!auto_login(&config(true, true), true, false));
        // nothing to sign in with
        assert!(!auto_login(&config(false, true), false, false));
        // off by default
        assert!(!auto_login(&config(false, false), true, false));
    }

    fn page(auto_login: bool, logged_out: bool) -> String {
        LoginPage {
            redirect_uri: Some("/frontend/user/me".to_owned()),
            oidc_data: Some(OidcProviderData {
                name: "Provider",
                redirect_url: "/frontend/login/oidc".to_owned(),
            }),
            allow_password_login: false,
            auto_login,
            logged_out,
        }
        .render()
        .unwrap()
    }

    #[test]
    fn test_login_page_submits_its_form_by_script() {
        let html = page(true, false);
        assert!(
            html.contains(
                r#"<script src="/frontend/assets/js/oidc_auto_login.js" defer></script>"#
            )
        );
        assert!(html.contains(
            r#"action="/frontend/login/oidc" method="post" id="form_login" data-auto-submit>"#
        ));
        // the flow still starts with the form's POST, and the target page is kept
        assert!(
            html.contains(r#"<input type="hidden" name="redirect_uri" value="/frontend/user/me">"#)
        );
        assert!(html.contains("Login with Provider"));
    }

    #[test]
    fn test_login_page_after_logout_shows_the_button_only() {
        let html = page(false, true);
        assert!(!html.contains("oidc_auto_login.js"));
        assert!(!html.contains("data-auto-submit"));
        assert!(html.contains("You have been logged out."));
        assert!(html.contains("Login with Provider"));
    }

    #[test]
    fn test_auto_login_script_submits_the_marked_form() {
        let script = include_str!("../../public/assets/js/oidc_auto_login.js");
        assert!(
            script.contains(r#"document.querySelector("form[data-auto-submit]")?.requestSubmit()"#)
        );
    }
}
