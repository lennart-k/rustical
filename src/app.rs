use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{web, App};
use rustical_auth::CheckAuthentication;
use rustical_frontend::configure_frontend;
use rustical_store::CalendarStore;
use std::sync::Arc;
use tokio::sync::RwLock;

pub fn make_app<CS: CalendarStore + ?Sized, A: CheckAuthentication>(
    cal_store: Arc<RwLock<CS>>,
    auth: Arc<A>,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    App::new()
        .wrap(Logger::new("[%s] %r"))
        .wrap(NormalizePath::trim())
        .service(web::scope("/caldav").configure(|cfg| {
            rustical_caldav::configure_dav(
                cfg,
                "/caldav".to_string(),
                auth.clone(),
                cal_store.clone(),
            )
        }))
        .service(web::scope("/carddav").configure(|cfg| {
            rustical_carddav::configure_dav(cfg, "/carddav".to_string(), auth.clone())
        }))
        .service(
            web::scope("/.well-known")
                .configure(|cfg| rustical_caldav::configure_well_known(cfg, "/caldav".to_string())), // .configure(|cfg| {
                                                                                                     //     rustical_carddav::configure_well_known(cfg, "/carddav".to_string())
                                                                                                     // }),
        )
        .service(
            web::scope("/frontend").configure(|cfg| configure_frontend(cfg, cal_store.clone())),
        )
        .service(web::redirect("/", "/frontend").permanent())
}
