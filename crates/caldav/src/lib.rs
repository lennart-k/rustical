use actix_web::http::Method;
use actix_web::web::{self, Data};
use actix_web::{guard, HttpResponse, Responder};
use calendar::resource::CalendarResource;
use event::resource::EventResource;
use principal::PrincipalResource;
use root::RootResource;
use rustical_auth::CheckAuthentication;
use rustical_dav::methods::{
    propfind::ServicePrefix, route_delete, route_propfind, route_proppatch,
};
use rustical_store::CalendarStore;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod calendar;
pub mod error;
pub mod event;
pub mod principal;
pub mod root;

pub use error::Error;

pub struct CalDavContext<C: CalendarStore + ?Sized> {
    pub store: Arc<RwLock<C>>,
}

pub fn configure_well_known(cfg: &mut web::ServiceConfig, caldav_root: String) {
    cfg.service(web::redirect("/caldav", caldav_root).permanent());
}

pub fn configure_dav<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    cfg: &mut web::ServiceConfig,
    prefix: String,
    auth: Arc<A>,
    store: Arc<RwLock<C>>,
) {
    let propfind_method = || web::method(Method::from_str("PROPFIND").unwrap());
    let proppatch_method = || web::method(Method::from_str("PROPPATCH").unwrap());
    let report_method = || web::method(Method::from_str("REPORT").unwrap());
    let mkcalendar_method = || web::method(Method::from_str("MKCALENDAR").unwrap());

    cfg.app_data(Data::new(CalDavContext {
        store: store.clone(),
    }))
    .app_data(Data::new(ServicePrefix(prefix)))
    .app_data(Data::from(store.clone()))
    .app_data(Data::from(auth))
    .service(
        web::resource("{path:.*}")
            // Without the guard this service would handle all requests
            .guard(guard::Method(Method::OPTIONS))
            .to(options_handler),
    )
    .service(
        web::resource("")
            .route(propfind_method().to(route_propfind::<A, RootResource>))
            .route(proppatch_method().to(route_proppatch::<A, RootResource>)),
    )
    .service(
        web::scope("/user").service(
            web::scope("/{principal}")
                .service(
                    web::resource("")
                        .route(propfind_method().to(route_propfind::<A, PrincipalResource<C>>))
                        .route(proppatch_method().to(route_proppatch::<A, PrincipalResource<C>>)),
                )
                .service(
                    web::scope("/{calendar}")
                        .service(
                            web::resource("")
                                .route(
                                    report_method().to(
                                        calendar::methods::report::route_report_calendar::<A, C>,
                                    ),
                                )
                                .route(
                                    propfind_method().to(route_propfind::<A, CalendarResource<C>>),
                                )
                                .route(
                                    proppatch_method()
                                        .to(route_proppatch::<A, CalendarResource<C>>),
                                )
                                .route(
                                    web::method(Method::DELETE)
                                        .to(route_delete::<A, CalendarResource<C>>),
                                )
                                .route(mkcalendar_method().to(
                                    calendar::methods::mkcalendar::route_mkcol_calendar::<A, C>,
                                )),
                        )
                        .service(
                            web::resource("/{event}")
                                .route(propfind_method().to(route_propfind::<A, EventResource<C>>))
                                .route(
                                    proppatch_method().to(route_proppatch::<A, EventResource<C>>),
                                )
                                .route(
                                    web::method(Method::DELETE)
                                        .to(route_delete::<A, EventResource<C>>),
                                )
                                .route(
                                    web::method(Method::GET).to(event::methods::get_event::<A, C>),
                                )
                                .route(
                                    web::method(Method::PUT).to(event::methods::put_event::<A, C>),
                                ),
                        ),
                ),
        ),
    );
}

async fn options_handler() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((
            "Allow",
            "OPTIONS, GET, HEAD, POST, PUT, REPORT, PROPFIND, PROPPATCH, MKCALENDAR",
        ))
        .insert_header((
            "DAV",
            "1, 2, 3, calendar-access, extended-mkcol",
            // "1, 2, 3, calendar-access, addressbook, extended-mkcol",
        ))
        .body("options")
}
