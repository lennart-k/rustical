use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::Key,
    http::Method,
    web::{self, Data, Path},
    Responder,
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
    user: User,
) -> impl Responder {
    let (owner, cal_id) = path.into_inner();
    CalendarPage {
        owner: owner.to_owned(),
        calendar: store.get_calendar(&owner, &cal_id).await.unwrap(),
    }
}

pub fn configure_frontend<AP: AuthenticationProvider, C: CalendarStore + ?Sized>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    store: Arc<C>,
    frontend_config: FrontendConfig,
) {
    cfg.service(
        web::scope("")
            .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
            .wrap(
                SessionMiddleware::builder(
                    CookieSessionStore::default(),
                    Key::from(&frontend_config.secret_key),
                )
                .cookie_secure(true)
                .cookie_content_security(actix_session::config::CookieContentSecurity::Private)
                .build(),
            )
            .app_data(Data::from(auth_provider))
            .app_data(Data::from(store.clone()))
            .service(EmbedService::<Assets>::new("/assets".to_owned()))
            .service(
                web::resource("/user/{user}").route(web::method(Method::GET).to(route_user::<C>)),
            )
            .service(
                web::resource("/user/{user}/{calendar}")
                    .route(web::method(Method::GET).to(route_calendar::<C>)),
            )
            .service(
                web::resource("/login")
                    .route(web::method(Method::GET).to(route_get_login))
                    .route(web::method(Method::POST).to(route_post_login::<AP>)),
            ),
    );
}
