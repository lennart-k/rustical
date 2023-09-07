use actix_web::{
    web::{self, Data},
    HttpResponse,
};
use rustical_store::calendar::CalendarStore;
use tokio::sync::RwLock;

pub fn configure_frontend<C: CalendarStore>(cfg: &mut web::ServiceConfig, store: Data<RwLock<C>>) {
    cfg.app_data(store);
}
