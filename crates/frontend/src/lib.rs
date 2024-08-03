use actix_web::{
    get,
    http::Method,
    web::{self, Data},
};
use askama::Template;
use rustical_store::CalendarStore;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[get("")]
async fn route_index() -> IndexTemplate {
    IndexTemplate {}
}

async fn route_user<C: CalendarStore + ?Sized>(store: Data<RwLock<C>>) -> IndexTemplate {
    IndexTemplate {}
}

pub fn configure_frontend<C: CalendarStore + ?Sized>(
    cfg: &mut web::ServiceConfig,
    store: Arc<RwLock<C>>,
) {
    cfg.app_data(Data::from(store.clone()))
        .service(route_index)
        .service(web::resource("/user/{user}").route(web::method(Method::GET).to(route_user::<C>)));
}
