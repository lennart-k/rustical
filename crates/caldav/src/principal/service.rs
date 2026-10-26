use crate::calendar::CalendarResourceService;
use crate::calendar::resource::{CalendarResource, objects_read_only};
use crate::principal::PrincipalResource;
use crate::{CalDavConfig, CalDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::Router;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_dav_push::DavPushStore;
use rustical_store::auth::{AuthenticationProvider, Principal};
use rustical_store::{CalendarStore, ShareStore, access};
use std::sync::Arc;

#[derive(Debug)]
pub struct PrincipalResourceService<AP: AuthenticationProvider, DP: DavPushStore, CS: CalendarStore>
{
    pub(crate) auth_provider: Arc<AP>,
    pub(crate) dav_push_store: Arc<DP>,
    pub(crate) cal_store: Arc<CS>,
    pub(crate) share_store: Arc<dyn ShareStore>,
    // If true only return the principal as the calendar home set, otherwise also groups
    pub(crate) simplified_home_set: bool,
    pub(crate) config: Arc<CalDavConfig>,
}

impl<AP: AuthenticationProvider, DP: DavPushStore, CS: CalendarStore> Clone
    for PrincipalResourceService<AP, DP, CS>
{
    fn clone(&self) -> Self {
        Self {
            auth_provider: self.auth_provider.clone(),
            dav_push_store: self.dav_push_store.clone(),
            cal_store: self.cal_store.clone(),
            share_store: self.share_store.clone(),
            simplified_home_set: self.simplified_home_set,
            config: self.config.clone(),
        }
    }
}

#[async_trait]
impl<AP: AuthenticationProvider, DP: DavPushStore, CS: CalendarStore> ResourceService
    for PrincipalResourceService<AP, DP, CS>
{
    type PathComponents = (String,);
    type MemberType = CalendarResource;
    type Resource = PrincipalResource;
    type Error = Error;
    type Principal = Principal;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, calendar-access";

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
            simplified_home_set: self.simplified_home_set,
        })
    }

    async fn get_members(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Vec<Self::MemberType>, Self::Error> {
        let vapid_pubkey = self.dav_push_store.get_vapid_pubkey_b64().await?;
        // The home's own calendars and the calendars shared with it
        let own = access::own_calendars(self.cal_store.as_ref(), principal)
            .await?
            .into_iter()
            .map(|(resolved, cal)| (cal.id.clone(), resolved, cal));
        let shared = access::shared_calendars(
            self.share_store.as_ref(),
            self.cal_store.as_ref(),
            principal,
        )
        .await?;
        Ok(own
            .chain(shared)
            .map(|(name, resolved, cal)| CalendarResource {
                read_only: objects_read_only(self.cal_store.as_ref(), &cal),
                cal,
                vapid_pubkey: vapid_pubkey.clone(),
                home: principal.to_owned(),
                name,
                resolved,
            })
            .collect())
    }

    fn axum_router<State: Send + Sync + Clone + 'static>(self) -> axum::Router<State> {
        Router::new()
            .nest(
                "/{calendar_id}",
                CalendarResourceService::new(
                    self.cal_store.clone(),
                    self.share_store.clone(),
                    self.dav_push_store.clone(),
                    self.config.clone(),
                )
                .axum_router(),
            )
            .route_service("/", self.axum_service())
    }
}

impl<AP: AuthenticationProvider, DP: DavPushStore, CS: CalendarStore> AxumMethods
    for PrincipalResourceService<AP, DP, CS>
{
}
