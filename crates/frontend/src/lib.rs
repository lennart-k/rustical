use actix_session::{
    SessionMiddleware, config::CookieContentSecurity, storage::CookieSessionStore,
};
use actix_web::{
    HttpRequest, HttpResponse, Responder,
    cookie::{Key, SameSite},
    dev::ServiceResponse,
    http::{Method, StatusCode},
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web::{self, Data, Path},
};
use askama::Template;
use askama_web::WebTemplate;
use assets::{Assets, EmbedService};
use oidc::{route_get_oidc, route_get_oidc_callback};
use routes::{
    addressbook::{route_addressbook, route_addressbook_restore},
    calendar::{route_calendar, route_calendar_restore},
    login::{route_get_login, route_post_login, route_post_logout},
};
use rustical_store::{
    Addressbook, AddressbookStore, Calendar, CalendarStore,
    auth::{AuthenticationMiddleware, AuthenticationProvider, User},
};
use std::sync::Arc;

mod assets;
mod config;
mod oidc;
mod routes;

pub use config::FrontendConfig;

#[derive(Template, WebTemplate)]
#[template(path = "pages/user.html")]
struct UserPage {
    pub user_id: String,
    pub calendars: Vec<Calendar>,
    pub deleted_calendars: Vec<Calendar>,
    pub addressbooks: Vec<Addressbook>,
    pub deleted_addressbooks: Vec<Addressbook>,
}

async fn route_user<CS: CalendarStore, AS: AddressbookStore>(
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
    dbg!(&user);

    let mut calendars = vec![];
    for group in user.memberships() {
        calendars.extend(cal_store.get_calendars(group).await.unwrap());
    }

    UserPage {
        calendars,
        deleted_calendars: cal_store.get_deleted_calendars(&user.id).await.unwrap(),
        addressbooks: addr_store.get_addressbooks(&user.id).await.unwrap(),
        deleted_addressbooks: addr_store.get_deleted_addressbooks(&user.id).await.unwrap(),
        user_id: user.id,
    }
    .respond_to(&req)
}

async fn route_root(user: Option<User>, req: HttpRequest) -> impl Responder {
    let redirect_url = match user {
        Some(user) => req
            .resource_map()
            .url_for(&req, "frontend_user", &[user.id])
            .unwrap(),
        None => req
            .resource_map()
            .url_for::<[_; 0], String>(&req, "frontend_login", [])
            .unwrap(),
    };
    web::Redirect::to(redirect_url.to_string()).permanent()
}

fn unauthorized_handler<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<B>> {
    let (req, _) = res.into_parts();
    let login_url = req.url_for_static("frontend_login").unwrap().to_string();

    let response = HttpResponse::Unauthorized().body(format!(
        r#"<!Doctype html>
        <html>
            <head>
                <meta http-equiv="refresh" content="2; url={login_url}" />
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

pub fn configure_frontend<AP: AuthenticationProvider, CS: CalendarStore, AS: AddressbookStore>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    cal_store: Arc<CS>,
    addr_store: Arc<AS>,
    frontend_config: FrontendConfig,
) {
    cfg.service(
        web::scope("")
            .wrap(ErrorHandlers::new().handler(StatusCode::UNAUTHORIZED, unauthorized_handler))
            .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(&frontend_config.secret_key),
                )
                .cookie_secure(true)
                .cookie_same_site(SameSite::Strict)
                .cookie_content_security(CookieContentSecurity::Private)
                .build(),
            )
            .app_data(Data::from(auth_provider))
            .app_data(Data::from(cal_store.clone()))
            .app_data(Data::from(addr_store.clone()))
            .app_data(Data::new(frontend_config.clone()))
            .service(EmbedService::<Assets>::new("/assets".to_owned()))
            .service(web::resource("").route(web::method(Method::GET).to(route_root)))
            .service(
                web::resource("/user/{user}")
                    .route(web::method(Method::GET).to(route_user::<CS, AS>))
                    .name("frontend_user"),
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
            )
            .service(
                web::resource("/login/oidc")
                    .name("frontend_login_oidc")
                    .route(web::method(Method::GET).to(route_get_oidc)),
            )
            .service(
                web::resource("/login/oidc/callback")
                    .name("frontend_oidc_callback")
                    .route(web::method(Method::GET).to(route_get_oidc_callback::<AP>)),
            ),
    );
}
