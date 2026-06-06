#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
use axum::{Extension, Router};
use derive_more::Constructor;
use http::Uri;
use principal::PrincipalResourceService;
use rustical_dav::resource::{PrincipalUri, ResourceService};
use rustical_dav::resources::RootResourceService;
use rustical_dav::rfc_3986_percent_encode;
use rustical_store::auth::middleware::AuthenticationLayer;
use rustical_store::auth::{AuthenticationProvider, Principal};
use rustical_store::{CalendarStore, SubscriptionStore};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod calendar;
pub mod calendar_object;
pub mod error;
pub mod principal;
pub use error::Error;

#[derive(Debug, Clone, Constructor)]
pub struct CalDavPrincipalUri(&'static str);

impl PrincipalUri for CalDavPrincipalUri {
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

pub fn caldav_router<AP: AuthenticationProvider, C: CalendarStore, S: SubscriptionStore>(
    prefix: &'static str,
    auth_provider: Arc<AP>,
    store: Arc<C>,
    subscription_store: Arc<S>,
    simplified_home_set: bool,
    config: Arc<CalDavConfig>,
) -> Router {
    Router::new().nest(
        prefix,
        RootResourceService::<_, Principal, CalDavPrincipalUri>::new(PrincipalResourceService {
            auth_provider: auth_provider.clone(),
            sub_store: subscription_store,
            cal_store: store,
            simplified_home_set,
            config,
        })
        .axum_router()
        .layer(AuthenticationLayer::new(auth_provider))
        .layer(Extension(CalDavPrincipalUri(prefix))),
    )
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields, default)]
pub struct CalDavConfig {
    #[serde(default = "default_true")]
    rfc7809: bool,
}

impl Default for CalDavConfig {
    fn default() -> Self {
        Self { rfc7809: true }
    }
}

#[cfg(test)]
mod tests {
    use crate::CalDavPrincipalUri;
    use rustical_dav::resource::PrincipalUri;

    #[rstest::rstest]
    #[case("user", "/caldav/principal/user/")]
    #[case("user with space", "/caldav/principal/user%20with%20space/")]
    #[case("asd@asd.de", "/caldav/principal/asd%40asd.de/")]
    fn test_principal_uri_encoding(#[case] principal: &str, #[case] output: &str) {
        assert_eq!(
            CalDavPrincipalUri("/caldav").principal_uri(principal),
            output
        );
    }
}
