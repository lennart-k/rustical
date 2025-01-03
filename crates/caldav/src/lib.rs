use actix_web::dev::ServiceResponse;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::{Method, StatusCode};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::web::{self, Data};
use actix_web::HttpResponse;
use calendar::resource::CalendarResourceService;
use calendar_object::resource::CalendarObjectResourceService;
use principal::{PrincipalResource, PrincipalResourceService};
use rustical_dav::resource::ResourceService;
use rustical_dav::resources::RootResourceService;
use rustical_store::auth::{AuthenticationMiddleware, AuthenticationProvider};
use rustical_store::CalendarStore;
use std::sync::Arc;

pub mod calendar;
pub mod calendar_object;
pub mod error;
pub mod principal;

pub use error::Error;

pub fn configure_well_known(cfg: &mut web::ServiceConfig, caldav_root: String) {
    cfg.service(web::redirect("/caldav", caldav_root).permanent());
}

pub fn configure_dav<AP: AuthenticationProvider, C: CalendarStore + ?Sized>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    store: Arc<C>,
) {
    cfg.service(
        web::scope("")
            .wrap(AuthenticationMiddleware::new(auth_provider))
            .wrap(
                ErrorHandlers::new().handler(StatusCode::METHOD_NOT_ALLOWED, |res| {
                    Ok(ErrorHandlerResponse::Response(
                        if res.request().method() == Method::OPTIONS {
                            let response = HttpResponse::Ok()
                                .insert_header((
                                    HeaderName::from_static("dav"),
                                    HeaderValue::from_static(
                                        "1, 2, 3, access-control, calendar-access, extended-mkcol",
                                    ),
                                ))
                                .finish();
                            ServiceResponse::new(res.into_parts().0, response).map_into_right_body()
                        } else {
                            res.map_into_left_body()
                        },
                    ))
                }),
            )
            .app_data(Data::from(store.clone()))
            .service(RootResourceService::<PrincipalResource>::actix_resource())
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
