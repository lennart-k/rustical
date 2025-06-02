use actix_web::HttpResponse;
use actix_web::body::BoxBody;
use actix_web::dev::{HttpServiceFactory, ServiceResponse};
use actix_web::http::header::{self, HeaderName, HeaderValue};
use actix_web::http::{Method, StatusCode};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::web::{self, Data};
use calendar::resource::CalendarResourceService;
use calendar_object::resource::CalendarObjectResourceService;
use calendar_set::CalendarSetResourceService;
use derive_more::Constructor;
use principal::{PrincipalResource, PrincipalResourceService};
use rustical_dav::resource::{PrincipalUri, ResourceService, ResourceServiceRoute};
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

#[derive(Debug, Clone, Constructor)]
pub struct CalDavPrincipalUri(&'static str);

impl PrincipalUri for CalDavPrincipalUri {
    fn principal_uri(&self, principal: &str) -> String {
        format!("{}/{}", self.0, principal)
    }
}

/// Quite a janky implementation but the default METHOD_NOT_ALLOWED response gives us the allowed
/// methods of a resource
fn options_handler() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::METHOD_NOT_ALLOWED, |res| {
        Ok(ErrorHandlerResponse::Response(
            if res.request().method() == Method::OPTIONS {
                let mut response = HttpResponse::Ok();
                response.insert_header((
                    HeaderName::from_static("dav"),
                    // https://datatracker.ietf.org/doc/html/rfc4918#section-18
                    HeaderValue::from_static(
                        "1, 3, access-control, calendar-access, extended-mkcol, calendar-no-timezone, webdav-push",
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
    })
}

pub fn caldav_service<
    AP: AuthenticationProvider,
    AS: AddressbookStore,
    C: CalendarStore,
    S: SubscriptionStore,
>(
    prefix: &'static str,
    auth_provider: Arc<AP>,
    store: Arc<C>,
    addr_store: Arc<AS>,
    subscription_store: Arc<S>,
) -> impl HttpServiceFactory {
    let birthday_store = Arc::new(ContactBirthdayStore::new(addr_store));

    web::scope("")
        .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
        .wrap(options_handler())
        .app_data(Data::from(store.clone()))
        .app_data(Data::from(birthday_store.clone()))
        .app_data(Data::from(subscription_store))
        .app_data(Data::new(CalDavPrincipalUri::new(
            format!("{prefix}/principal").leak(),
        )))
        .service(
            RootResourceService::<PrincipalResource, User, CalDavPrincipalUri>::default()
                .actix_resource(),
        )
        .service(
            web::scope("/principal").service(
                web::scope("/{principal}")
                    .service(
                        PrincipalResourceService {
                            auth_provider,
                            home_set: &[("calendar", false), ("birthdays", true)],
                        }
                        .actix_resource(),
                    )
                    .service(
                        web::scope("/calendar")
                            .service(
                                CalendarSetResourceService::new(store.clone()).actix_resource(),
                            )
                            .service(
                                web::scope("/{calendar_id}")
                                    .service(ResourceServiceRoute(
                                        CalendarResourceService::<_, S>::new(store.clone()),
                                    ))
                                    .service(
                                        web::scope("/{object_id}.ics").service(
                                            CalendarObjectResourceService::new(store.clone())
                                                .actix_resource(),
                                        ),
                                    ),
                            ),
                    )
                    .service(
                        web::scope("/birthdays")
                            .service(
                                CalendarSetResourceService::new(birthday_store.clone())
                                    .actix_resource(),
                            )
                            .service(
                                web::scope("/{calendar_id}")
                                    .service(ResourceServiceRoute(
                                        CalendarResourceService::<_, S>::new(
                                            birthday_store.clone(),
                                        ),
                                    ))
                                    .service(
                                        web::scope("/{object_id}.ics").service(
                                            CalendarObjectResourceService::new(
                                                birthday_store.clone(),
                                            )
                                            .actix_resource(),
                                        ),
                                    ),
                            ),
                    ),
            ),
        )
        .service(subscription_resource::<S>())
}
