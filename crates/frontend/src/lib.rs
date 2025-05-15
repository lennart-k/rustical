use actix_session::{
    SessionMiddleware,
    config::CookieContentSecurity,
    storage::{CookieSessionStore, SessionStore},
};
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    cookie::{Key, SameSite},
    dev::ServiceResponse,
    http::{Method, StatusCode, header},
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web::{self, Data, Form, Path, Redirect},
};
use askama::Template;
use askama_web::WebTemplate;
use assets::{Assets, EmbedService};
use async_trait::async_trait;
use rand::{Rng, distributions::Alphanumeric};
use routes::{
    addressbook::{route_addressbook, route_addressbook_restore},
    calendar::{route_calendar, route_calendar_restore},
    login::{route_get_login, route_post_login, route_post_logout},
};
use rustical_oidc::{OidcConfig, OidcServiceConfig, UserStore, configure_oidc};
use rustical_store::{
    Addressbook, AddressbookStore, Calendar, CalendarStore,
    auth::{AuthenticationMiddleware, AuthenticationProvider, User, user::AppToken},
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

mod assets;
mod config;
pub mod nextcloud_login;
mod routes;

pub const ROUTE_NAME_HOME: &str = "frontend_home";
pub const ROUTE_USER_NAMED: &str = "frontend_user_named";

pub use config::{FrontendConfig, generate_frontend_secret};

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
    pub app_tokens: Vec<AppToken>,
    pub calendars: Vec<Calendar>,
    pub deleted_calendars: Vec<Calendar>,
    pub addressbooks: Vec<Addressbook>,
    pub deleted_addressbooks: Vec<Addressbook>,
    pub is_apple: bool,
}

async fn route_user_named<CS: CalendarStore, AS: AddressbookStore, AP: AuthenticationProvider>(
    path: Path<String>,
    cal_store: Data<CS>,
    addr_store: Data<AS>,
    auth_provider: Data<AP>,
    user: User,
    req: HttpRequest,
) -> impl Responder {
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

    let is_apple = req
        .headers()
        .get(header::USER_AGENT)
        .and_then(|user_agent| user_agent.to_str().ok())
        .map(|ua| ua.contains("Apple") || ua.contains("Mac OS"))
        .unwrap_or_default();

    UserPage {
        app_tokens: auth_provider.get_app_tokens(&user.id).await.unwrap(),
        calendars,
        deleted_calendars,
        addressbooks,
        deleted_addressbooks,
        user,
        is_apple,
    }
    .respond_to(&req)
}

async fn route_get_home(user: User, req: HttpRequest) -> Redirect {
    Redirect::to(
        req.url_for(ROUTE_USER_NAMED, &[user.id])
            .unwrap()
            .to_string(),
    )
    .see_other()
}

async fn route_root(user: Option<User>, req: HttpRequest) -> impl Responder {
    let redirect_url = match user {
        Some(_) => req.url_for_static(ROUTE_NAME_HOME).unwrap(),
        None => req
            .resource_map()
            .url_for::<[_; 0], String>(&req, "frontend_login", [])
            .unwrap(),
    };
    web::Redirect::to(redirect_url.to_string()).permanent()
}

#[derive(Template)]
#[template(path = "apple_configuration/template.xml")]
pub struct AppleConfig {
    token_name: String,
    account_description: String,
    hostname: String,
    caldav_principal_url: String,
    carddav_principal_url: String,
    user: String,
    token: String,
    caldav_profile_uuid: Uuid,
    carddav_profile_uuid: Uuid,
    plist_uuid: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PostAppTokenForm {
    name: String,
    #[serde(default)]
    apple: bool,
}

async fn route_post_app_token<AP: AuthenticationProvider>(
    user: User,
    auth_provider: Data<AP>,
    path: Path<String>,
    Form(PostAppTokenForm { apple, name }): Form<PostAppTokenForm>,
    req: HttpRequest,
) -> Result<HttpResponse, rustical_store::Error> {
    assert!(!name.is_empty());
    assert_eq!(path.into_inner(), user.id);
    let token = generate_app_token();
    auth_provider
        .add_app_token(&user.id, name.to_owned(), token.clone())
        .await?;
    if apple {
        let hostname = req.full_url().host_str().unwrap().to_owned();
        let profile = AppleConfig {
            token_name: name,
            account_description: format!("{}@{}", &user.id, &hostname),
            hostname,
            caldav_principal_url: req
                .url_for("caldav_principal", [&user.id])
                .unwrap()
                .to_string(),
            carddav_principal_url: req
                .url_for("carddav_principal", [&user.id])
                .unwrap()
                .to_string(),
            user: user.id.to_owned(),
            token,
            caldav_profile_uuid: Uuid::new_v4(),
            carddav_profile_uuid: Uuid::new_v4(),
            plist_uuid: Uuid::new_v4(),
        }
        .render()
        .unwrap();
        Ok(HttpResponse::Ok()
            .insert_header(header::ContentDisposition::attachment(format!(
                "rustical-{}.mobileconfig",
                user.id
            )))
            .insert_header((
                header::CONTENT_TYPE,
                "application/x-apple-aspen-config; charset=utf-8",
            ))
            .body(profile))
    } else {
        Ok(HttpResponse::Ok().body(token))
    }
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
        .app_data(Data::from(auth_provider.clone()))
        .app_data(Data::from(cal_store.clone()))
        .app_data(Data::from(addr_store.clone()))
        .app_data(Data::new(frontend_config.clone()))
        .app_data(Data::new(oidc_config.clone()))
        .service(EmbedService::<Assets>::new("/assets".to_owned()))
        .service(web::resource("").route(web::method(Method::GET).to(route_root)))
        .service(
            web::resource("/user")
                .get(route_get_home)
                .name(ROUTE_NAME_HOME),
        )
        .service(
            web::resource("/user/{user}")
                .get(route_user_named::<CS, AS, AP>)
                .name(ROUTE_USER_NAMED),
        )
        // App token management
        .service(web::resource("/user/{user}/app_token").post(route_post_app_token::<AP>))
        .service(
            // POST because HTML5 forms don't support DELETE method
            web::resource("/user/{user}/app_token/{id}/delete").post(route_delete_app_token::<AP>),
        )
        // Calendar
        .service(web::resource("/user/{user}/calendar/{calendar}").get(route_calendar::<CS>))
        .service(
            web::resource("/user/{user}/calendar/{calendar}/restore")
                .post(route_calendar_restore::<CS>),
        )
        // Addressbook
        .service(
            web::resource("/user/{user}/addressbook/{addressbook}").get(route_addressbook::<AS>),
        )
        .service(
            web::resource("/user/{user}/addressbook/{addressbook}/restore")
                .post(route_addressbook_restore::<AS>),
        )
        // Login
        .service(
            web::resource("/login")
                .name("frontend_login")
                .get(route_get_login)
                .post(route_post_login::<AP>),
        )
        .service(
            web::resource("/logout")
                .name("frontend_logout")
                .post(route_post_logout),
        );

    if let Some(oidc_config) = oidc_config {
        scope = scope.service(web::scope("/login/oidc").configure(|cfg| {
            configure_oidc(
                cfg,
                oidc_config,
                OidcServiceConfig {
                    default_redirect_route_name: ROUTE_NAME_HOME,
                    session_key_user_id: "user",
                },
                Arc::new(OidcUserStore(auth_provider.clone())),
            )
        }));
    }

    cfg.service(scope);
}

struct OidcUserStore<AP: AuthenticationProvider>(Arc<AP>);

#[async_trait(?Send)]
impl<AP: AuthenticationProvider> UserStore for OidcUserStore<AP> {
    type Error = rustical_store::Error;

    async fn user_exists(&self, id: &str) -> Result<bool, Self::Error> {
        Ok(self.0.get_principal(id).await?.is_some())
    }

    async fn insert_user(&self, id: &str) -> Result<(), Self::Error> {
        self.0
            .insert_principal(
                User {
                    id: id.to_owned(),
                    displayname: None,
                    principal_type: Default::default(),
                    password: None,
                    memberships: vec![],
                },
                false,
            )
            .await
    }
}
