use actix_web::{
    HttpResponse,
    body::BoxBody,
    dev::{HttpServiceFactory, ServiceResponse},
    http::{
        Method, StatusCode,
        header::{self, HeaderName, HeaderValue},
    },
    middleware::{ErrorHandlerResponse, ErrorHandlers},
    web::Data,
};
use derive_more::Constructor;
pub use error::Error;
use principal::PrincipalResourceService;
use rustical_dav::resource::{PrincipalUri, ResourceService};
use rustical_dav::resources::RootResourceService;
use rustical_store::{
    AddressbookStore, SubscriptionStore,
    auth::{AuthenticationMiddleware, AuthenticationProvider, User},
};
use std::sync::Arc;

pub mod address_object;
pub mod addressbook;
pub mod error;
pub mod principal;

#[derive(Debug, Clone, Constructor)]
pub struct CardDavPrincipalUri(&'static str);

impl PrincipalUri for CardDavPrincipalUri {
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
                        "1, 3, access-control, addressbook, extended-mkcol, webdav-push",
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

pub fn carddav_service<AP: AuthenticationProvider, A: AddressbookStore, S: SubscriptionStore>(
    prefix: &'static str,
    auth_provider: Arc<AP>,
    store: Arc<A>,
    subscription_store: Arc<S>,
) -> impl HttpServiceFactory {
    RootResourceService::<_, User, CardDavPrincipalUri>::new(
        PrincipalResourceService::<_, _, S>::new(
            store.clone(),
            auth_provider.clone(),
            subscription_store.clone(),
        ),
    )
    .actix_scope()
    .wrap(AuthenticationMiddleware::new(auth_provider.clone()))
    .wrap(options_handler())
    .app_data(Data::from(store.clone()))
    .app_data(Data::new(CardDavPrincipalUri::new(
        format!("{prefix}/principal").leak(),
    )))
    // TODO: Add endpoint to delete subscriptions
}
