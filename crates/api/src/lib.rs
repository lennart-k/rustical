use actix_web::{
    http::Method,
    web::{self, Data, Path},
    Responder,
};
use rustical_store::calendar::CalendarStore;
use tokio::sync::RwLock;

pub fn configure_api<C: CalendarStore + ?Sized>(
    cfg: &mut web::ServiceConfig,
    store: Data<RwLock<C>>,
) {
    cfg.app_data(store).route(
        "/{cid}/events",
        web::method(Method::GET).to(get_events::<C>),
    );
}

pub async fn get_events<C: CalendarStore + ?Sized>(
    store: Data<RwLock<C>>,
    path: Path<String>,
) -> impl Responder {
    let cid = path.into_inner();
    let events = store.read().await.get_events(&cid).await.unwrap();
    serde_json::to_string_pretty(&events)
}
