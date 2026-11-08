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
use rustical_dav_push::DavPushStore;
use rustical_store::auth::middleware::AuthenticationLayer;
use rustical_store::{
    AddressbookStore,
    auth::{AuthenticationProvider, Principal},
};
use serde::{Deserialize, Serialize};
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

const fn default_true() -> bool {
    true
}

/// `CardDAV` server options.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct CardDavConfig {
    /// Advertise vCard 4.0 in `supported-address-data`.
    ///
    /// Set to `false` when macOS Contacts shares the address book. Apple's
    /// client rejects vCard 4.0 and fails the whole sync; `DAVx5` writes 4.0
    /// whenever the server advertises it. RFC 6352 only requires accepting
    /// advertised versions, so 4.0 PUTs still work.
    #[serde(default = "default_true")]
    pub advertise_vcard4: bool,
}

impl Default for CardDavConfig {
    fn default() -> Self {
        Self {
            advertise_vcard4: true,
        }
    }
}

pub fn carddav_router<AP: AuthenticationProvider, A: AddressbookStore, DP: DavPushStore>(
    prefix: &'static str,
    auth_provider: Arc<AP>,
    store: Arc<A>,
    dav_push_store: Arc<DP>,
    config: Arc<CardDavConfig>,
) -> Router {
    let principal_service =
        PrincipalResourceService::new(store, auth_provider.clone(), dav_push_store, config);
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

    #[test]
    fn carddav_config_defaults_to_advertising_vcard4() {
        assert!(crate::CardDavConfig::default().advertise_vcard4);
    }
}
