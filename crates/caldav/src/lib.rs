use actix_web::HttpResponse;
use actix_web::dev::{HttpServiceFactory, ServiceResponse};
use actix_web::http::header::{self, HeaderName, HeaderValue};
use actix_web::http::{Method, StatusCode};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::web::{self, Data};
use calendar::resource::CalendarResourceService;
use calendar_object::resource::CalendarObjectResourceService;
use calendar_set::CalendarSetResourceService;
use principal::{PrincipalResource, PrincipalResourceService};
use rustical_dav::resource::{NamedRoute, ResourceService, ResourceServiceRoute};
use rustical_dav::resources::RootResourceService;
use rustical_store::auth::{AuthenticationMiddleware, AuthenticationProvider, User};
use rustical_store::{AddressbookStore, CalendarStore, ContactBirthdayStore, SubscriptionStore};
use std::sync::Arc;
use subscription::subscription_resource;

pub mod calendar;
pub mod calendar_object;
pub mod calendar_set;
pub mod error;
pub mod principal;
mod subscription;

pub use error::Error;

pub fn caldav_service<
    AP: AuthenticationProvider,
    AS: AddressbookStore,
    C: CalendarStore,
    S: SubscriptionStore,
>(
    auth_provider: Arc<AP>,
    store: Arc<C>,
    addr_store: Arc<AS>,
    subscription_store: Arc<S>,
) -> impl HttpServiceFactory {
    let birthday_store = Arc::new(ContactBirthdayStore::new(addr_store));

    web::scope("")
            .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
            .wrap(
                ErrorHandlers::new().handler(StatusCode::METHOD_NOT_ALLOWED, |res| {
                    Ok(ErrorHandlerResponse::Response(
                        if res.request().method() == Method::OPTIONS {
                            let mut response = HttpResponse::Ok();
                            response.insert_header((
                                HeaderName::from_static("dav"),
                                // https://datatracker.ietf.org/doc/html/rfc4918#section-18
                                HeaderValue::from_static(
                                    "1, 3, access-control, calendar-access, extended-mkcol, calendar-no-timezone",
                                ),
                            ));

                            if let Some(allow) = res.headers().get(header::ALLOW) {
                                response.insert_header((header::ALLOW, allow.to_owned()));
                            }
                            ServiceResponse::new(res.into_parts().0, response.finish()).map_into_right_body()
                        } else {
                            res.map_into_left_body()
                        },
                    ))
                }),
            )
            .app_data(Data::from(store.clone()))
            .app_data(Data::from(birthday_store.clone()))
            .app_data(Data::from(subscription_store))
            .service(RootResourceService::<PrincipalResource, User>::default().actix_resource())
            .service(
                web::scope("/principal").service(
                    web::scope("/{principal}")
                        .service(PrincipalResourceService{auth_provider, home_set: &[
                            ("calendar", false), ("birthdays", true)
                        ]}.actix_resource().name(PrincipalResource::route_name()))
                        .service(web::scope("/calendar")
                            .service(CalendarSetResourceService::new(store.clone()).actix_resource())
                            .service(
                                web::scope("/{calendar}")
                                    .service(
                                        ResourceServiceRoute(CalendarResourceService::<_, S>::new(store.clone()))
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
                                        ResourceServiceRoute(CalendarResourceService::<_, S>::new(birthday_store.clone()))
                                    )
                                        .service(web::scope("/{object}").service(CalendarObjectResourceService::new(birthday_store.clone()).actix_resource()
                                    ))
                            )
                        )
                ),
            ).service(subscription_resource::<S>())
}
