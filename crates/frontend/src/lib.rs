use actix_session::{
    SessionMiddleware,
    config::CookieContentSecurity,
    storage::{CookieSessionStore, SessionStore},
};
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    cookie::{Key, SameSite},
    dev::ServiceResponse,
    http::{Method, StatusCode},
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web::{self, Data, Form, Path, Redirect},
};
use askama::Template;
use askama_web::WebTemplate;
use assets::{Assets, EmbedService};
use oidc::{route_get_oidc_callback, route_post_oidc};
use rand::{Rng, distributions::Alphanumeric};
use routes::{
    addressbook::{route_addressbook, route_addressbook_restore},
    calendar::{route_calendar, route_calendar_restore},
    login::{route_get_login, route_post_login, route_post_logout},
};
use rustical_store::{
    Addressbook, AddressbookStore, Calendar, CalendarStore,
    auth::{AuthenticationMiddleware, AuthenticationProvider, User},
};
use serde::Deserialize;
use std::sync::Arc;

mod assets;
mod config;
pub mod nextcloud_login;
mod oidc;
mod routes;

pub use config::{FrontendConfig, OidcConfig};

pub fn generate_app_token() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .map(char::from)
        .take(64)
        .collect()
}

#[derive(Template, WebTemplate)]
#[template(path = "pages/user.html")]
struct UserPage {
    pub user: User,
    pub calendars: Vec<Calendar>,
    pub deleted_calendars: Vec<Calendar>,
    pub addressbooks: Vec<Addressbook>,
    pub deleted_addressbooks: Vec<Addressbook>,
}

async fn route_user(user: User, req: HttpRequest) -> Redirect {
    Redirect::to(
        req.url_for("frontend_user_named", &[user.id])
            .unwrap()
            .to_string(),
    )
    .see_other()
}

async fn route_user_named<CS: CalendarStore, AS: AddressbookStore>(
    path: Path<String>,
    cal_store: Data<CS>,
    addr_store: Data<AS>,
    user: User,
    req: HttpRequest,
) -> impl Responder {
    // TODO: Check for authorization
    let user_id = path.into_inner();
    if user_id != user.id {
        return actix_web::HttpResponse::Unauthorized().body("Unauthorized");
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

    UserPage {
        calendars,
        deleted_calendars,
        addressbooks,
        deleted_addressbooks,
        user,
    }
    .respond_to(&req)
}

async fn route_root(user: Option<User>, req: HttpRequest) -> impl Responder {
    let redirect_url = match user {
        Some(_) => req.url_for_static("frontend_user").unwrap(),
        None => req
            .resource_map()
            .url_for::<[_; 0], String>(&req, "frontend_login", [])
            .unwrap(),
    };
    web::Redirect::to(redirect_url.to_string()).permanent()
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PostAppTokenForm {
    name: String,
}

async fn route_post_app_token<AP: AuthenticationProvider>(
    user: User,
    auth_provider: Data<AP>,
    path: Path<String>,
    Form(PostAppTokenForm { name }): Form<PostAppTokenForm>,
) -> Result<impl Responder, rustical_store::Error> {
    assert!(!name.is_empty());
    assert_eq!(path.into_inner(), user.id);
    let token = generate_app_token();
    auth_provider
        .add_app_token(&user.id, name, token.clone())
        .await?;
    Ok(token)
}

async fn route_delete_app_token<AP: AuthenticationProvider>(
    user: User,
    auth_provider: Data<AP>,
    path: Path<(String, String)>,
) -> Result<Redirect, rustical_store::Error> {
    let (path_user, token_id) = path.into_inner();
    assert_eq!(path_user, user.id);
    auth_provider.remove_app_token(&user.id, &token_id).await?;
    Ok(Redirect::to("/frontend/user").see_other())
}

pub(crate) fn unauthorized_handler<B>(
    res: ServiceResponse<B>,
) -> actix_web::Result<ErrorHandlerResponse<B>> {
    let (req, _) = res.into_parts();
    let redirect_uri = req.uri().to_string();
    let mut login_url = req.url_for_static("frontend_login").unwrap();
    login_url
        .query_pairs_mut()
        .append_pair("redirect_uri", &redirect_uri);
    let login_url = login_url.to_string();

    let response = HttpResponse::Unauthorized().body(format!(
        r#"<!Doctype html>
        <html>
            <head>
                <meta http-equiv="refresh" content="1; url={login_url}" />
            </head>
            <body>
                Unauthorized, redirecting to <a href="{login_url}">login page</a>
            </body>
        <html>
    "#
    ));

    let res = ServiceResponse::new(req, response)
        .map_into_boxed_body()
        .map_into_right_body();

    Ok(ErrorHandlerResponse::Response(res))
}

pub fn session_middleware(frontend_secret: [u8; 64]) -> SessionMiddleware<impl SessionStore> {
    SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&frontend_secret))
        .cookie_secure(true)
        .cookie_same_site(SameSite::Strict)
        .cookie_content_security(CookieContentSecurity::Private)
        .build()
}

pub fn configure_frontend<AP: AuthenticationProvider, CS: CalendarStore, AS: AddressbookStore>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    cal_store: Arc<CS>,
    addr_store: Arc<AS>,
    frontend_config: FrontendConfig,
    oidc_config: Option<OidcConfig>,
) {
    let mut scope = web::scope("")
        .wrap(ErrorHandlers::new().handler(StatusCode::UNAUTHORIZED, unauthorized_handler))
        .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
        .wrap(session_middleware(frontend_config.secret_key))
        .app_data(Data::from(auth_provider))
        .app_data(Data::from(cal_store.clone()))
        .app_data(Data::from(addr_store.clone()))
        .app_data(Data::new(frontend_config.clone()))
        .app_data(Data::new(oidc_config.clone()))
        .service(EmbedService::<Assets>::new("/assets".to_owned()))
        .service(web::resource("").route(web::method(Method::GET).to(route_root)))
        .service(
            web::resource("/user")
                .route(web::method(Method::GET).to(route_user))
                .name("frontend_user"),
        )
        .service(
            web::resource("/user/{user}")
                .route(web::method(Method::GET).to(route_user_named::<CS, AS>))
                .name("frontend_user_named"),
        )
        .service(
            web::resource("/user/{user}/app_token")
                .route(web::method(Method::POST).to(route_post_app_token::<AP>)),
        )
        .service(
            web::resource("/user/{user}/app_token/{id}/delete")
                .route(web::method(Method::POST).to(route_delete_app_token::<AP>)),
        )
        .service(
            web::resource("/user/{user}/calendar/{calendar}")
                .route(web::method(Method::GET).to(route_calendar::<CS>)),
        )
        .service(
            web::resource("/user/{user}/calendar/{calendar}/restore")
                .route(web::method(Method::POST).to(route_calendar_restore::<CS>)),
        )
        .service(
            web::resource("/user/{user}/addressbook/{addressbook}")
                .route(web::method(Method::GET).to(route_addressbook::<AS>)),
        )
        .service(
            web::resource("/user/{user}/addressbook/{addressbook}/restore")
                .route(web::method(Method::POST).to(route_addressbook_restore::<AS>)),
        )
        .service(
            web::resource("/login")
                .name("frontend_login")
                .route(web::method(Method::GET).to(route_get_login))
                .route(web::method(Method::POST).to(route_post_login::<AP>)),
        )
        .service(
            web::resource("/logout")
                .name("frontend_logout")
                .route(web::method(Method::POST).to(route_post_logout)),
        );

    if let Some(oidc_config) = oidc_config {
        scope = scope
            .app_data(Data::new(oidc_config))
            .service(
                web::resource("/login/oidc")
                    .name("frontend_login_oidc")
                    .route(web::method(Method::POST).to(route_post_oidc)),
            )
            .service(
                web::resource("/login/oidc/callback")
                    .name("frontend_oidc_callback")
                    .route(web::method(Method::GET).to(route_get_oidc_callback::<AP>)),
            );
    }

    cfg.service(scope);
}
