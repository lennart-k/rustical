use std::sync::Arc;

use crate::pages::user::{Section, UserPage};
use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension,
    extract::Path,
    response::{IntoResponse, Redirect},
};
use axum_extra::TypedHeader;
use headers::{HeaderMapExt, Host, UserAgent};
use http::{HeaderMap, StatusCode};
use rustical_store::auth::{AppToken, AuthenticationProvider, Principal};

impl Section for ProfileSection {
    fn name() -> &'static str {
        "profile"
    }
}

#[derive(Template, WebTemplate)]
#[template(path = "components/sections/profile_section.html")]
pub struct ProfileSection {
    pub user: Principal,
    pub app_tokens: Vec<AppToken>,
    pub is_apple: bool,
    pub davx5_hostname: Option<String>,
}

pub async fn route_user_named<AP: AuthenticationProvider>(
    Path(user_id): Path<String>,
    Extension(auth_provider): Extension<Arc<AP>>,
    TypedHeader(host): TypedHeader<Host>,
    user: Principal,
    headers: HeaderMap,
) -> impl IntoResponse {
    if user_id != user.id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let ua = headers.typed_get::<UserAgent>();
    let is_apple = ua
        .as_ref()
        .is_some_and(|ua| ua.as_str().contains("Apple") || ua.as_str().contains("Mac OS"));
    let davx5_hostname =
        ua.and_then(|ua| ua.as_str().contains("Android").then_some(host.to_string()));

    UserPage {
        section: ProfileSection {
            user: user.clone(),
            app_tokens: auth_provider.get_app_tokens(&user.id).await.unwrap(),
            is_apple,
            davx5_hostname,
        },
        user,
    }
    .into_response()
}

pub async fn route_get_home(user: Principal) -> Redirect {
    Redirect::to(&format!("/frontend/user/{}", user.id))
}

pub async fn route_root(user: Option<Principal>) -> Redirect {
    match user {
        Some(user) => route_get_home(user).await,
        None => Redirect::to("/frontend/login"),
    }
}
