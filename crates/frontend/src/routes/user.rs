use std::sync::Arc;

use askama::Template;
use askama_web::WebTemplate;
use axum::{
    Extension,
    extract::Path,
    response::{IntoResponse, Redirect},
};
use axum_extra::{TypedHeader, extract::Host};
use headers::UserAgent;
use http::StatusCode;
use rustical_store::auth::{AppToken, AuthenticationProvider, Principal};

use crate::pages::user::{Section, UserPage};

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
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Host(host): Host,
    user: Principal,
) -> impl IntoResponse {
    if user_id != user.id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let is_apple = user_agent.as_str().contains("Apple") || user_agent.as_str().contains("Mac OS");
    let davx5_hostname = user_agent.as_str().contains("Android").then_some(host);

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
