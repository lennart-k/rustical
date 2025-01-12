use actix_web::dev::ServiceResponse;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::{Method, StatusCode};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::web::{self, Data};
use actix_web::HttpResponse;
use calendar::resource::CalendarResourceService;
use calendar_object::resource::CalendarObjectResourceService;
use calendar_set::CalendarSetResourceService;
use principal::{PrincipalResource, PrincipalResourceService};
use rustical_dav::resource::{NamedRoute, ResourceService, ResourceServiceRoute};
use rustical_dav::resources::RootResourceService;
use rustical_store::auth::{AuthenticationMiddleware, AuthenticationProvider};
use rustical_store::{AddressbookStore, CalendarStore, ContactBirthdayStore};
use std::sync::Arc;

pub mod calendar;
pub mod calendar_object;
pub mod calendar_set;
pub mod error;
pub mod principal;

pub use error::Error;

pub fn configure_well_known(cfg: &mut web::ServiceConfig, caldav_root: String) {
    cfg.service(web::redirect("/caldav", caldav_root).permanent());
}

pub fn configure_dav<
    AP: AuthenticationProvider,
    AS: AddressbookStore + ?Sized,
    C: CalendarStore + ?Sized,
>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    store: Arc<C>,
    addr_store: Arc<AS>,
) {
    let birthday_store = Arc::new(ContactBirthdayStore::new(addr_store));
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
                                    // https://datatracker.ietf.org/doc/html/rfc4918#section-18
                                    HeaderValue::from_static(
                                        "1, 3, access-control, calendar-access, extended-mkcol, calendar-no-timezone",
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
            .app_data(Data::from(birthday_store.clone()))
            .service(RootResourceService::<PrincipalResource>::default().actix_resource())
            .service(
                web::scope("/user").service(
                    web::scope("/{principal}")
                        .service(PrincipalResourceService(&[
                            "calendar", "birthdays"
                        ]).actix_resource().name(PrincipalResource::route_name()))
                        .service(web::scope("/calendar")
                            .service(CalendarSetResourceService::new(store.clone()).actix_resource())
                            .service(
                                web::scope("/{calendar}")
                                    .service(
                                        ResourceServiceRoute(CalendarResourceService::new(store.clone()))
                                    )
                                        .service(web::scope("/{object}").service(CalendarObjectResourceService::new(store.clone()).actix_resource()
                                    ))
                            )
                        )
                        .service(web::scope("/birthdays")
                            .service(CalendarSetResourceService::new(birthday_store.clone()).actix_resource())
                            .service(
                                web::scope("/{calendar}")
                                    .service(
                                        ResourceServiceRoute(CalendarResourceService::new(birthday_store.clone()))
                                    )
                                        .service(web::scope("/{object}").service(CalendarObjectResourceService::new(birthday_store.clone()).actix_resource()
                                    ))
                            )
                        )
                ),
            ),
    );
}
