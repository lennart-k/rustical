use std::sync::Arc;

use axum::{response::Redirect, routing::get, Router};
use rustical_caldav::caldav_app;
use rustical_store::{
    auth::AuthenticationProvider, AddressbookStore, CalendarStore, SubscriptionStore,
};

pub fn make_app<AS: AddressbookStore, CS: CalendarStore, S: SubscriptionStore>(
    addr_store: Arc<AS>,
    cal_store: Arc<CS>,
    subscription_store: Arc<S>,
    auth_provider: Arc<impl AuthenticationProvider>,
) -> Router {
    Router::new()
        .nest(
            "/caldav",
            caldav_app(auth_provider, cal_store, addr_store, subscription_store),
        )
        .route(
            "/.well-known/caldav",
            get(|| async { Redirect::permanent("/caldav") }),
        )
        .route(
            "/.well-known/carddav",
            get(|| async { Redirect::permanent("/carddav") }),
        )
}

// pub fn make_app<AS: AddressbookStore, CS: CalendarStore, S: SubscriptionStore>(
//     addr_store: Arc<AS>,
//     cal_store: Arc<CS>,
//     subscription_store: Arc<S>,
//     auth_provider: Arc<impl AuthenticationProvider>,
//     frontend_config: FrontendConfig,
//     nextcloud_login_config: NextcloudLoginConfig,
//     nextcloud_flows_state: Arc<NextcloudFlows>,
// ) -> App<
//     impl ServiceFactory<
//         ServiceRequest,
//         Response = ServiceResponse<impl MessageBody>,
//         Config = (),
//         InitError = (),
//         Error = actix_web::Error,
//     >,
// > {
//     let mut app = App::new()
//         // .wrap(Logger::new("[%s] %r"))
//         .wrap(TracingLogger::default())
//         .wrap(NormalizePath::trim())
//         .service(web::scope("/caldav").service(caldav_service(
//             auth_provider.clone(),
//             cal_store.clone(),
//             addr_store.clone(),
//             subscription_store.clone(),
//         )))
//         .service(web::scope("/carddav").service(carddav_service(
//             auth_provider.clone(),
//             addr_store.clone(),
//             subscription_store,
//         )))
//         .service(
//             web::scope("/.well-known")
//                 .service(web::redirect("/caldav", "/caldav"))
//                 .service(web::redirect("/carddav", "/carddav")),
//         );
//
//     if frontend_config.enabled {
//         app = app
//             .service(web::scope("/frontend").configure(|cfg| {
//                 configure_frontend(
//                     cfg,
//                     auth_provider.clone(),
//                     cal_store.clone(),
//                     addr_store.clone(),
//                     frontend_config,
//                 )
//             }))
//             .service(web::redirect("/", "/frontend").see_other());
//     }
//     if nextcloud_login_config.enabled {
//         app = app.configure(|cfg| {
//             configure_nextcloud_login(cfg, nextcloud_flows_state, auth_provider.clone())
//         });
//     }
//     app
// }
