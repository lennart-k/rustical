use actix_web::body::MessageBody;
use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::middleware::NormalizePath;
use actix_web::{web, App};
use rustical_caldav::caldav_service;
use rustical_carddav::carddav_service;
use rustical_frontend::{configure_frontend, FrontendConfig};
use rustical_store::auth::AuthenticationProvider;
use rustical_store::{AddressbookStore, CalendarStore, SubscriptionStore};
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

pub fn make_app<AS: AddressbookStore, CS: CalendarStore, S: SubscriptionStore>(
    addr_store: Arc<AS>,
    cal_store: Arc<CS>,
    subscription_store: Arc<S>,
    auth_provider: Arc<impl AuthenticationProvider>,
    frontend_config: FrontendConfig,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = actix_web::Error,
    >,
> {
    let mut app = App::new()
        // .wrap(Logger::new("[%s] %r"))
        .wrap(TracingLogger::default())
        .wrap(NormalizePath::trim())
        .service(web::scope("/caldav").service(caldav_service(
            auth_provider.clone(),
            cal_store.clone(),
            addr_store.clone(),
            subscription_store.clone(),
        )))
        .service(web::scope("/carddav").service(carddav_service(
            auth_provider.clone(),
            addr_store.clone(),
            subscription_store,
        )))
        .service(
            web::scope("/.well-known")
                .service(web::redirect("/caldav", "/caldav"))
                .service(web::redirect("/carddav", "/carddav")),
        );

    if frontend_config.enabled {
        app = app
            .service(web::scope("/frontend").configure(|cfg| {
                configure_frontend(
                    cfg,
                    auth_provider.clone(),
                    cal_store.clone(),
                    addr_store.clone(),
                    frontend_config,
                )
            }))
            .service(web::redirect("/", "/frontend").see_other());
    }
    app
}
