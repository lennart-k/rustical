use axum::response::Redirect;
use axum::routing::any;
use axum::{Extension, Router};
use derive_more::Constructor;
pub use error::Error;
use principal::PrincipalResourceService;
use rustical_dav::resource::{PrincipalUri, ResourceService};
use rustical_dav::resources::RootResourceService;
use rustical_store::auth::middleware::AuthenticationLayer;
use rustical_store::{
    AddressbookStore, SubscriptionStore,
    auth::{AuthenticationProvider, User},
};
use std::sync::Arc;

pub mod address_object;
pub mod addressbook;
pub mod error;
pub mod principal;

#[derive(Debug, Clone, Constructor)]
pub struct CardDavPrincipalUri(&'static str);

impl PrincipalUri for CardDavPrincipalUri {
    fn principal_collection(&self) -> String {
        format!("{}/principal/", self.0)
    }
    fn principal_uri(&self, principal: &str) -> String {
        format!("{}{}/", self.principal_collection(), principal)
    }
}

pub fn carddav_router<AP: AuthenticationProvider, A: AddressbookStore, S: SubscriptionStore>(
    prefix: &'static str,
    auth_provider: Arc<AP>,
    store: Arc<A>,
    subscription_store: Arc<S>,
) -> Router {
    let principal_service = PrincipalResourceService::new(
        store.clone(),
        auth_provider.clone(),
        subscription_store.clone(),
    );
    Router::new()
        .nest(
            prefix,
            RootResourceService::<_, User, CardDavPrincipalUri>::new(principal_service.clone())
                .axum_router()
                .layer(AuthenticationLayer::new(auth_provider))
                .layer(Extension(CardDavPrincipalUri(prefix))),
        )
        .route(
            "/.well-known/carddav",
            any(async || Redirect::permanent(prefix)),
        )
}
