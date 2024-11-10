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
use assets::{Assets, EmbedService};
use routes::login::{route_get_login, route_post_login};
use rustical_store::{
    auth::{AuthenticationMiddleware, AuthenticationProvider, User},
    Calendar, CalendarStore,
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
}

async fn route_user<C: CalendarStore + ?Sized>(
    path: Path<String>,
    store: Data<C>,
    user: User,
) -> impl Responder {
    let user_id = path.into_inner();
    UserPage {
        calendars: store.get_calendars(&user.id).await.unwrap(),
        user_id: user.id,
    }
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

pub fn configure_frontend<AP: AuthenticationProvider, C: CalendarStore + ?Sized>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    store: Arc<C>,
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
            .app_data(Data::from(store.clone()))
            .service(EmbedService::<Assets>::new("/assets".to_owned()))
            .service(web::resource("").route(web::method(Method::GET).to(route_root)))
            .service(
                web::resource("/user/{user}")
                    .route(web::method(Method::GET).to(route_user::<C>))
                    .name("frontend_user"),
            )
            .service(
                web::resource("/user/{user}/{calendar}")
                    .route(web::method(Method::GET).to(route_calendar::<C>)),
            )
            .service(
                web::resource("/login")
                    .name("frontend_login")
                    .route(web::method(Method::GET).to(route_get_login))
                    .route(web::method(Method::POST).to(route_post_login::<AP>)),
            ),
    );
}
