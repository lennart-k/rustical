use actix_web::{
    dev::ServiceResponse,
    http::{
        header::{HeaderName, HeaderValue},
        Method, StatusCode,
    },
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web::{self, Data},
    HttpResponse,
};
use address_object::resource::AddressObjectResourceService;
use addressbook::resource::AddressbookResourceService;
pub use error::Error;
use principal::{PrincipalResource, PrincipalResourceService};
use rustical_dav::resource::{NamedRoute, ResourceService};
use rustical_dav::resources::RootResourceService;
use rustical_store::{
    auth::{AuthenticationMiddleware, AuthenticationProvider},
    AddressbookStore, SubscriptionStore,
};
use std::sync::Arc;

pub mod address_object;
pub mod addressbook;
pub mod error;
pub mod principal;

pub fn configure_dav<AP: AuthenticationProvider, A: AddressbookStore, S: SubscriptionStore>(
    cfg: &mut web::ServiceConfig,
    auth_provider: Arc<AP>,
    store: Arc<A>,
    subscription_store: Arc<S>,
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
                                    // https://datatracker.ietf.org/doc/html/rfc4918#section-18
                                    HeaderValue::from_static(
                                        "1, 3, access-control, addressbook, extended-mkcol",
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
            .app_data(Data::from(subscription_store))
            .service(RootResourceService::<PrincipalResource>::default().actix_resource())
            .service(
                web::scope("/user").service(
                    web::scope("/{principal}")
                        .service(
                            PrincipalResourceService::new(store.clone())
                                .actix_resource()
                                .name(PrincipalResource::route_name()),
                        )
                        .service(
                            web::scope("/{addressbook}")
                                .service(
                                    AddressbookResourceService::<A, S>::new(store.clone())
                                        .actix_resource(),
                                )
                                .service(
                                    web::scope("/{object}").service(
                                        AddressObjectResourceService::<A>::new(store.clone())
                                            .actix_resource(),
                                    ),
                                ),
                        ),
                ),
            ),
    );
}
