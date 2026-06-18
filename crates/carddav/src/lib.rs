#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
use axum::response::Redirect;
use axum::routing::any;
use axum::{Extension, Router};
use derive_more::Constructor;
pub use error::Error;
use http::Uri;
use principal::PrincipalResourceService;
use rustical_dav::resource::{PrincipalUri, ResourceService};
use rustical_dav::resources::RootResourceService;
use rustical_dav::rfc_3986_percent_encode;
use rustical_store::auth::middleware::AuthenticationLayer;
use rustical_store::{
    AddressbookStore, SubscriptionStore,
    auth::{AuthenticationProvider, Principal},
};
use std::sync::Arc;

pub mod address_object;
pub mod addressbook;
pub mod error;
pub mod principal;

#[derive(Debug, Clone, Constructor)]
pub struct CardDavPrincipalUri(&'static str);

impl PrincipalUri for CardDavPrincipalUri {
    fn principal_collection(&self) -> Uri {
        Uri::builder()
            .path_and_query(format!("{}/principal/", self.0))
            .build()
            .unwrap()
    }
    fn principal_uri(&self, principal: &str) -> Uri {
        let principal = rfc_3986_percent_encode(principal);
        Uri::builder()
            .path_and_query(format!("{}{}/", self.principal_collection(), principal))
            .build()
            .unwrap()
    }
}

pub fn carddav_router<AP: AuthenticationProvider, A: AddressbookStore, S: SubscriptionStore>(
    prefix: &'static str,
    auth_provider: Arc<AP>,
    store: Arc<A>,
    subscription_store: Arc<S>,
    vapid_public_key: Option<&'static str>,
) -> Router {
    let principal_service = PrincipalResourceService::new(
        store,
        auth_provider.clone(),
        subscription_store,
        vapid_public_key,
    );
    Router::new()
        .nest(
            prefix,
            RootResourceService::<_, Principal, CardDavPrincipalUri>::new(principal_service)
                .axum_router()
                .layer(AuthenticationLayer::new(auth_provider))
                .layer(Extension(CardDavPrincipalUri(prefix))),
        )
        .route(
            "/.well-known/carddav",
            any(async || Redirect::permanent(prefix)),
        )
}

#[cfg(test)]
mod tests {
    use crate::CardDavPrincipalUri;
    use rustical_dav::resource::PrincipalUri;

    #[rstest::rstest]
    #[case("user", "/carddav/principal/user/")]
    #[case("user with space", "/carddav/principal/user%20with%20space/")]
    #[case("asd@asd.de", "/carddav/principal/asd%40asd.de/")]
    fn test_principal_uri_encoding(#[case] principal: &str, #[case] output: &str) {
        assert_eq!(
            CardDavPrincipalUri("/carddav").principal_uri(principal),
            output
        );
    }
}
