use crate::calendar::CalendarResourceService;
use crate::calendar::resource::CalendarResource;
use crate::principal::PrincipalResource;
use crate::{CalDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::Router;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_store::auth::{AuthenticationProvider, Principal};
use rustical_store::{CalendarStore, SubscriptionStore};
use std::sync::Arc;

#[derive(Debug)]
pub struct PrincipalResourceService<
    AP: AuthenticationProvider,
    S: SubscriptionStore,
    CS: CalendarStore,
> {
    pub(crate) auth_provider: Arc<AP>,
    pub(crate) sub_store: Arc<S>,
    pub(crate) cal_store: Arc<CS>,
    // If true only return the principal as the calendar home set, otherwise also groups
    pub(crate) simplified_home_set: bool,
}

impl<AP: AuthenticationProvider, S: SubscriptionStore, CS: CalendarStore> Clone
    for PrincipalResourceService<AP, S, CS>
{
    fn clone(&self) -> Self {
        Self {
            auth_provider: self.auth_provider.clone(),
            sub_store: self.sub_store.clone(),
            cal_store: self.cal_store.clone(),
            simplified_home_set: self.simplified_home_set,
        }
    }
}

#[async_trait]
impl<AP: AuthenticationProvider, S: SubscriptionStore, CS: CalendarStore> ResourceService
    for PrincipalResourceService<AP, S, CS>
{
    type PathComponents = (String,);
    type MemberType = CalendarResource;
    type Resource = PrincipalResource;
    type Error = Error;
    type Principal = Principal;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, calendar-access, calendar-proxy";

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
        let calendars = self.cal_store.get_calendars(principal).await?;

        Ok(calendars
            .into_iter()
            .map(|cal| CalendarResource {
                read_only: self.cal_store.is_read_only(&cal.id),
                cal,
            })
            .collect())
    }

    fn axum_router<State: Send + Sync + Clone + 'static>(self) -> axum::Router<State> {
        Router::new()
            .nest(
                "/{calendar_id}",
                CalendarResourceService::new(self.cal_store.clone(), self.sub_store.clone())
                    .axum_router(),
            )
            .route_service("/", self.axum_service())
    }
}

impl<AP: AuthenticationProvider, S: SubscriptionStore, CS: CalendarStore> AxumMethods
    for PrincipalResourceService<AP, S, CS>
{
}
