use actix_session::{
    config::CookieContentSecurity, storage::CookieSessionStore, SessionMiddleware,
};
use actix_web::{
    cookie::{Key, SameSite},
    dev::ServiceResponse,
    http::{Method, StatusCode},
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web::{self, Data, Path},
    HttpRequest, HttpResponse, Responder,
};
use askama::Template;
use askama_actix::TemplateToResponse;
use assets::{Assets, EmbedService};
use routes::login::{route_get_login, route_post_login};
use rustical_store::{
    auth::{AuthenticationMiddleware, AuthenticationProvider, User},
    Addressbook, AddressbookStore, Calendar, CalendarStore,
};
use std::sync::Arc;

mod assets;
mod config;
mod routes;

pub use config::FrontendConfig;

#[derive(Template)]
#[template(path = "pages/user.html")]
struct UserPage {
    pub user_id: String,
    pub calendars: Vec<Calendar>,
    pub deleted_calendars: Vec<Calendar>,
    pub addressbooks: Vec<Addressbook>,
    pub deleted_addressbooks: Vec<Addressbook>,
}

async fn route_user<CS: CalendarStore + ?Sized, AS: AddressbookStore + ?Sized>(
    path: Path<String>,
    cal_store: Data<CS>,
    addr_store: Data<AS>,
    user: User,
) -> impl Responder {
    // TODO: Check for authorization
    let user_id = path.into_inner();
    if user_id != user.id {
        return actix_web::HttpResponse::Unauthorized().body("Unauthorized");
    }

    UserPage {
        calendars: cal_store.get_calendars(&user.id).await.unwrap(),
        deleted_calendars: cal_store.get_deleted_calendars(&user.id).await.unwrap(),
        addressbooks: addr_store.get_addressbooks(&user.id).await.unwrap(),
        deleted_addressbooks: addr_store.get_deleted_addressbooks(&user.id).await.unwrap(),
        user_id: user.id,
    }
    .to_response()
}

#[derive(Template)]
#[template(path = "pages/calendar.html")]
struct CalendarPage {
    owner: String,
    calendar: Calendar,
}

async fn route_calendar<C: CalendarStore + ?Sized>(
    path: Path<(String, String)>,
    store: Data<C>,
    _user: User,
) -> impl Responder {
    let (owner, cal_id) = path.into_inner();
    CalendarPage {
        owner: owner.to_owned(),
        calendar: store.get_calendar(&owner, &cal_id).await.unwrap(),
    }
}

#[derive(Template)]
#[template(path = "pages/addressbook.html")]
struct AddressbookPage {
    owner: String,
    addressbook: Addressbook,
}

async fn route_addressbook<AS: AddressbookStore + ?Sized>(
    path: Path<(String, String)>,
    store: Data<AS>,
    _user: User,
) -> impl Responder {
    let (owner, addrbook_id) = path.into_inner();
    AddressbookPage {
        owner: owner.to_owned(),
        addressbook: store.get_addressbook(&owner, &addrbook_id).await.unwrap(),
    }
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

    // let response = Redirect::to(login_url).respond_to(&req);
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

pub fn configure_frontend<
    AP: AuthenticationProvider,
    CS: CalendarStore + ?Sized,
    AS: AddressbookStore + ?Sized,
>(
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
                web::resource("/user/{user}/addressbook/{calendar}")
                    .route(web::method(Method::GET).to(route_addressbook::<AS>)),
            )
            .service(
                web::resource("/login")
                    .name("frontend_login")
                    .route(web::method(Method::GET).to(route_get_login))
                    .route(web::method(Method::POST).to(route_post_login::<AP>)),
            ),
    );
}
