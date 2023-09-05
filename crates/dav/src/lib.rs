use actix_web::http::Method;
use actix_web::web::{self, Data};
use actix_web::{guard, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use error::Error;
use routes::{calendar, event, principal, root};
use rustical_store::calendar::CalendarStore;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod depth_extractor;
pub mod error;
pub mod namespace;
mod propfind;
pub mod routes;

pub struct Context<C: CalendarStore> {
    pub prefix: String,
    pub store: Arc<RwLock<C>>,
}

pub fn configure_well_known(cfg: &mut web::ServiceConfig, caldav_root: String) {
    cfg.service(web::redirect("/caldav", caldav_root).permanent());
}

pub fn configure_dav<C: CalendarStore>(
    cfg: &mut web::ServiceConfig,
    prefix: String,
    store: Arc<RwLock<C>>,
) {
    let propfind_method = || Method::from_str("PROPFIND").unwrap();
    let report_method = || Method::from_str("REPORT").unwrap();

    let auth = HttpAuthentication::basic(|req, creds| async move {
        if creds.user_id().is_empty() {
            // not authenticated
            Err((actix_web::error::ErrorUnauthorized("Unauthorized"), req))
        } else {
            Ok(req)
        }
    });

    cfg.app_data(Data::new(Context { prefix, store }))
        .service(
            web::resource("{path:.*}")
                // Without the guard this service would handle all requests
                .guard(guard::Method(Method::OPTIONS))
                .to(options_handler),
        )
        .service(
            web::resource("")
                .route(web::method(propfind_method()).to(root::route_propfind_root::<C>))
                .wrap(auth.clone()),
        )
        .service(
            web::resource("/{principal}")
                .route(web::method(propfind_method()).to(principal::route_propfind_principal::<C>))
                .wrap(auth.clone()),
        )
        .service(
            web::resource("/{principal}/{calendar}")
                .route(web::method(report_method()).to(calendar::route_report_calendar::<C>))
                .route(web::method(propfind_method()).to(calendar::route_propfind_calendar::<C>))
                .wrap(auth.clone()),
        )
        .service(
            web::resource("/{principal}/{calendar}/{event}")
                .route(web::method(Method::DELETE).to(event::delete_event::<C>))
                .route(web::method(Method::GET).to(event::get_event::<C>))
                .route(web::method(Method::PUT).to(event::put_event::<C>))
                .wrap(auth.clone()),
        );
}

async fn options_handler() -> impl Responder {
    HttpResponse::Ok()
        .insert_header((
            "Allow",
            "OPTIONS, GET, HEAD, POST, PUT, REPORT, PROPFIND, PROPPATCH",
        ))
        .insert_header((
            "DAV",
            "1, 2, 3, calendar-access, extended-mkcol",
            // "1, 2, 3, calendar-access, addressbook, extended-mkcol",
        ))
        .body("options")
}
