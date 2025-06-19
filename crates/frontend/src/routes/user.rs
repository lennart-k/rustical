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
use rustical_store::{
    Addressbook, AddressbookStore, Calendar, CalendarStore,
    auth::{AppToken, AuthenticationProvider, Principal},
};

#[derive(Template, WebTemplate)]
#[template(path = "pages/user.html")]
pub struct UserPage {
    pub user: Principal,
    pub app_tokens: Vec<AppToken>,
    pub calendars: Vec<Calendar>,
    pub deleted_calendars: Vec<Calendar>,
    pub addressbooks: Vec<Addressbook>,
    pub deleted_addressbooks: Vec<Addressbook>,
    pub is_apple: bool,
    pub davx5_hostname: Option<String>,
}

pub async fn route_user_named<
    CS: CalendarStore,
    AS: AddressbookStore,
    AP: AuthenticationProvider,
>(
    Path(user_id): Path<String>,
    Extension(cal_store): Extension<Arc<CS>>,
    Extension(addr_store): Extension<Arc<AS>>,
    Extension(auth_provider): Extension<Arc<AP>>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    Host(host): Host,
    user: Principal,
) -> impl IntoResponse {
    if user_id != user.id {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let mut calendars = vec![];
    for group in user.memberships() {
        calendars.extend(cal_store.get_calendars(group).await.unwrap());
    }

    let mut deleted_calendars = vec![];
    for group in user.memberships() {
        deleted_calendars.extend(cal_store.get_deleted_calendars(group).await.unwrap());
    }

    let mut addressbooks = vec![];
    for group in user.memberships() {
        addressbooks.extend(addr_store.get_addressbooks(group).await.unwrap());
    }

    let mut deleted_addressbooks = vec![];
    for group in user.memberships() {
        deleted_addressbooks.extend(addr_store.get_deleted_addressbooks(group).await.unwrap());
    }

    let is_apple = user_agent.as_str().contains("Apple") || user_agent.as_str().contains("Mac OS");
    let davx5_hostname = user_agent.as_str().contains("Android").then_some(host);

    UserPage {
        app_tokens: auth_provider.get_app_tokens(&user.id).await.unwrap(),
        calendars,
        deleted_calendars,
        addressbooks,
        deleted_addressbooks,
        user,
        is_apple,
        davx5_hostname,
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
