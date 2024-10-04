use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{
    cookie::Key,
    get,
    http::Method,
    web::{self, Data, Path},
    Responder,
};
use askama::Template;
use login::{route_get_login, route_post_login};
use rustical_store::{
    auth::{AuthenticationMiddleware, AuthenticationProvider, User},
    model::Calendar,
    CalendarStore,
};
use std::sync::Arc;
use tokio::sync::RwLock;

mod config;
mod login;

pub use config::FrontendConfig;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[get("")]
async fn route_index(session: Session) -> IndexTemplate {
    if let Some(user) = session.get::<User>("user").unwrap() {
        dbg!(user);
    } else {
        session.insert("user", "lennart").unwrap();
    }
    dbg!(session.status());
    IndexTemplate {}
}

#[derive(Template)]
#[template(path = "components/calendar_list.html")]
struct CalendarList {
    pub owner: String,
    pub calendars: Vec<Calendar>,
}

#[derive(Template)]
#[template(path = "layouts/default.html")]
struct DefaultTemplate<Body: Template> {
    pub body: Body,
}

async fn route_user<C: CalendarStore + ?Sized>(
    path: Path<String>,
    store: Data<RwLock<C>>,
) -> impl Responder {
    let store = store.read().await;
    let owner = path.into_inner();
    DefaultTemplate {
        body: CalendarList {
            owner: owner.to_owned(),
            calendars: store.get_calendars(&owner).await.unwrap(),
        },
    }
}

pub fn configure_frontend<AP: AuthenticationProvider, C: CalendarStore + ?Sized>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    store: Arc<RwLock<C>>,
) {
    cfg.service(
        web::scope("")
            .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&[0; 64]))
                    .cookie_secure(true)
                    .cookie_content_security(actix_session::config::CookieContentSecurity::Private)
                    .build(),
            )
            .app_data(Data::from(auth_provider))
            .app_data(Data::from(store.clone()))
            .service(actix_files::Files::new("/public", "crates/frontend/public").prefer_utf8(true))
            .service(route_index)
            .service(
                web::resource("/user/{user}").route(web::method(Method::GET).to(route_user::<C>)),
            )
            .service(
                web::resource("/login")
                    .route(web::method(Method::GET).to(route_get_login))
                    .route(web::method(Method::POST).to(route_post_login::<AP>)),
            ),
    );
}
