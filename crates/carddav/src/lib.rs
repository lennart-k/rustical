use crate::address_object::resource::AddressObjectResourceService;
use crate::addressbook::resource::AddressbookResourceService;
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
    fn principal_uri(&self, principal: &str) -> String {
        format!("{}/principal/{}", self.0, principal)
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
        .route_service(
            "/",
            RootResourceService::<_, User, CardDavPrincipalUri>::new(principal_service.clone())
                .axum_service(),
        )
        .route_service("/principal/{principal}", principal_service.axum_service())
        .route_service(
            "/principal/{principal}/{addressbook_id}",
            AddressbookResourceService::new(store.clone(), subscription_store.clone())
                .axum_service(),
        )
        .route_service(
            "/principal/{principal}/{addressbook_id}/{object_id}",
            AddressObjectResourceService::new(store.clone()).axum_service(),
        )
        .layer(AuthenticationLayer::new(auth_provider))
        .layer(Extension(CardDavPrincipalUri(prefix)))
}
