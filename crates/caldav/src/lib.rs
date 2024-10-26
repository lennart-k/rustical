use actix_web::dev::Service;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::{Method, StatusCode};
use actix_web::web::{self, Data};
use calendar::resource::CalendarResourceService;
use calendar_object::resource::CalendarObjectResourceService;
use futures_util::FutureExt;
use principal::PrincipalResourceService;
use root::RootResourceService;
use rustical_dav::resource::ResourceService;
use rustical_store::auth::{AuthenticationMiddleware, AuthenticationProvider};
use rustical_store::CalendarStore;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod calendar;
pub mod calendar_object;
pub mod error;
pub mod principal;
pub mod root;

pub use error::Error;

pub fn configure_well_known(cfg: &mut web::ServiceConfig, caldav_root: String) {
    cfg.service(web::redirect("/caldav", caldav_root).permanent());
}

pub fn configure_dav<AP: AuthenticationProvider, C: CalendarStore + ?Sized>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    store: Arc<RwLock<C>>,
) {
    cfg.service(
        web::scope("")
            .wrap(AuthenticationMiddleware::new(auth_provider))
            .wrap_fn(|req, srv| {
                // Middleware to set the DAV header
                // Could be more elegant if actix_web::guard::RegisteredMethods was public :(
                let method = req.method().clone();
                srv.call(req).map(move |res| {
                    if method == Method::OPTIONS {
                        return res.map(|mut response| {
                            if response.status() == StatusCode::METHOD_NOT_ALLOWED {
                                response.headers_mut().insert(
                                    HeaderName::from_static("dav"),
                                    HeaderValue::from_static(
                                        "1, 2, 3, calendar-access, extended-mkcol",
                                    ),
                                );
                                *response.response_mut().status_mut() = StatusCode::OK;
                            }
                            response
                        });
                    }
                    res
                })
            })
            .app_data(Data::from(store.clone()))
            .service(RootResourceService::actix_resource())
            .service(
                web::scope("/user").service(
                    web::scope("/{principal}")
                        .service(PrincipalResourceService::<C>::actix_resource())
                        .service(
                            web::scope("/{calendar}")
                                .service(CalendarResourceService::<C>::actix_resource())
                                .service(
                                    web::scope("/{object}").service(
                                        CalendarObjectResourceService::<C>::actix_resource(),
                                    ),
                                ),
                        ),
                ),
            ),
    );
}
