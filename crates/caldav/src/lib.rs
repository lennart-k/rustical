use axum::Router;
use principal::PrincipalResource;
use rustical_dav::resource::ResourceService;
use rustical_dav::resources::RootResourceService;
use rustical_store::auth::AuthenticationProvider;
use rustical_store::{AddressbookStore, CalendarStore, SubscriptionStore};
use std::sync::Arc;

pub mod calendar;
pub mod calendar_object;
pub mod calendar_set;
pub mod error;
pub mod principal;
// mod subscription;

pub use error::Error;

pub fn caldav_app<
    AP: AuthenticationProvider,
    AS: AddressbookStore,
    C: CalendarStore,
    S: SubscriptionStore,
>(
    auth_provider: Arc<AP>,
    store: Arc<C>,
    addr_store: Arc<AS>,
    subscription_store: Arc<S>,
) -> Router {
    Router::new().route_service(
        "/",
        RootResourceService::<PrincipalResource>::default().axum_service(auth_provider),
    )
}

// pub fn caldav_service<
//     AP: AuthenticationProvider,
//     AS: AddressbookStore,
//     C: CalendarStore,
//     S: SubscriptionStore,
// >(
//     auth_provider: Arc<AP>,
//     store: Arc<C>,
//     addr_store: Arc<AS>,
//     subscription_store: Arc<S>,
// ) -> impl HttpServiceFactory {
//     let birthday_store = Arc::new(ContactBirthdayStore::new(addr_store));
//
//     web::scope("")
// .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
// .wrap(
//     ErrorHandlers::new().handler(StatusCode::METHOD_NOT_ALLOWED, |res| {
//         Ok(ErrorHandlerResponse::Response(
//             if res.request().method() == Method::OPTIONS {
//                 let response = HttpResponse::Ok()
//                     .insert_header((
//                         HeaderName::from_static("dav"),
//                         // https://datatracker.ietf.org/doc/html/rfc4918#section-18
//                         HeaderValue::from_static(
//                             "1, 3, access-control, calendar-access, extended-mkcol, calendar-no-timezone",
//                         ),
//                     ))
//                     .finish();
//                 ServiceResponse::new(res.into_parts().0, response).map_into_right_body()
//             } else {
//                 res.map_into_left_body()
//             },
//         ))
//     }),
// )
// .app_data(Data::from(store.clone()))
// .app_data(Data::from(birthday_store.clone()))
// .app_data(Data::from(subscription_store))
// .service(RootResourceService::<PrincipalResource>::default().actix_resource())
// .service(
//     web::scope("/principal").service(
//         web::scope("/{principal}")
//             .service(PrincipalResourceService{auth_provider, home_set: &[
//                 ("calendar", false), ("birthdays", true)
//             ]}.actix_resource().name(PrincipalResource::route_name()))
//             .service(web::scope("/calendar")
//                 .service(CalendarSetResourceService::new(store.clone()).actix_resource())
//                 .service(
//                     web::scope("/{calendar}")
//                         .service(
//                             ResourceServiceRoute(CalendarResourceService::<_, S>::new(store.clone()))
//                         )
//                             .service(web::scope("/{object}").service(CalendarObjectResourceService::new(store.clone()).actix_resource()
//                         ))
//                 )
//             )
//             .service(web::scope("/birthdays")
//                 .service(CalendarSetResourceService::new(birthday_store.clone()).actix_resource())
//                 .service(
//                     web::scope("/{calendar}")
//                         .service(
//                             ResourceServiceRoute(CalendarResourceService::<_, S>::new(birthday_store.clone()))
//                         )
//                             .service(web::scope("/{object}").service(CalendarObjectResourceService::new(birthday_store.clone()).actix_resource()
//                         ))
//                 )
//             )
//     ),
// ).service(subscription_resource::<S>())
// }
