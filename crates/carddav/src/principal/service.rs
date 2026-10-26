use crate::CardDavPrincipalUri;
use crate::Error;
use crate::addressbook::AddressbookResourceService;
use crate::addressbook::resource::AddressbookResource;
use crate::principal::PrincipalResource;
use async_trait::async_trait;
use axum::Router;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_dav_push::DavPushStore;
use rustical_store::auth::{AuthenticationProvider, Principal};
use rustical_store::{AddressbookStore, ShareStore, access};
use std::sync::Arc;

pub struct PrincipalResourceService<
    A: AddressbookStore,
    AP: AuthenticationProvider,
    DP: DavPushStore,
> {
    addr_store: Arc<A>,
    share_store: Arc<dyn ShareStore>,
    auth_provider: Arc<AP>,
    dav_push_store: Arc<DP>,
}

impl<A: AddressbookStore, AP: AuthenticationProvider, DP: DavPushStore> Clone
    for PrincipalResourceService<A, AP, DP>
{
    fn clone(&self) -> Self {
        Self {
            addr_store: self.addr_store.clone(),
            share_store: self.share_store.clone(),
            auth_provider: self.auth_provider.clone(),
            dav_push_store: self.dav_push_store.clone(),
        }
    }
}

impl<A: AddressbookStore, AP: AuthenticationProvider, DP: DavPushStore>
    PrincipalResourceService<A, AP, DP>
{
    pub fn new(
        addr_store: Arc<A>,
        share_store: Arc<dyn ShareStore>,
        auth_provider: Arc<AP>,
        dav_push_store: Arc<DP>,
    ) -> Self {
        Self {
            addr_store,
            share_store,
            auth_provider,
            dav_push_store,
        }
    }
}

#[async_trait]
impl<A: AddressbookStore, AP: AuthenticationProvider, DP: DavPushStore> ResourceService
    for PrincipalResourceService<A, AP, DP>
{
    type PathComponents = (String,);
    type MemberType = AddressbookResource;
    type Resource = PrincipalResource;
    type Error = Error;
    type Principal = Principal;
    type PrincipalUri = CardDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, addressbook";

    async fn get_resource(
        &self,
        (principal,): &Self::PathComponents,
        _show_deleted: bool,
    ) -> Result<Self::Resource, Self::Error> {
        let user = self
            .auth_provider
            .get_principal(principal)
            .await?
            .ok_or(crate::Error::NotFound)?;
        Ok(PrincipalResource {
            members: self.auth_provider.list_members(&user.id).await?,
            principal: user,
        })
    }

    async fn get_members(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Vec<Self::MemberType>, Self::Error> {
        let vapid_pubkey = self.dav_push_store.get_vapid_pubkey_b64().await?;
        // The home's own addressbooks and the addressbooks shared with it
        let own = access::own_addressbooks(self.addr_store.as_ref(), principal)
            .await?
            .into_iter()
            .map(|(resolved, addressbook)| (addressbook.id.clone(), resolved, addressbook));
        let shared = access::shared_addressbooks(
            self.share_store.as_ref(),
            self.addr_store.as_ref(),
            principal,
        )
        .await?;
        Ok(own
            .chain(shared)
            .map(|(name, resolved, addressbook)| AddressbookResource {
                addressbook,
                vapid_pubkey: vapid_pubkey.clone(),
                home: principal.to_owned(),
                name,
                resolved,
            })
            .collect())
    }

    fn axum_router<State: Send + Sync + Clone + 'static>(self) -> Router<State> {
        Router::new()
            .nest(
                "/{addressbook_id}",
                AddressbookResourceService::new(
                    self.addr_store.clone(),
                    self.share_store.clone(),
                    self.dav_push_store.clone(),
                )
                .axum_router(),
            )
            .route_service("/", self.axum_service())
    }
}

impl<A: AddressbookStore, AP: AuthenticationProvider, DP: DavPushStore> AxumMethods
    for PrincipalResourceService<A, AP, DP>
{
}
