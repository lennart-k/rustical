use crate::calendar_set::{CalendarSetResource, CalendarSetResourceService};
use crate::principal::PrincipalResource;
use crate::{CalDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::Router;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_store::auth::{AuthenticationProvider, User};
use rustical_store::{CalendarStore, SubscriptionStore};
use std::sync::Arc;

#[derive(Debug)]
pub struct PrincipalResourceService<
    AP: AuthenticationProvider,
    S: SubscriptionStore,
    CS: CalendarStore,
    BS: CalendarStore,
> {
    pub(crate) auth_provider: Arc<AP>,
    pub(crate) sub_store: Arc<S>,
    pub(crate) cal_store: Arc<CS>,
    pub(crate) birthday_store: Arc<BS>,
}

impl<AP: AuthenticationProvider, S: SubscriptionStore, CS: CalendarStore, BS: CalendarStore> Clone
    for PrincipalResourceService<AP, S, CS, BS>
{
    fn clone(&self) -> Self {
        Self {
            auth_provider: self.auth_provider.clone(),
            sub_store: self.sub_store.clone(),
            cal_store: self.cal_store.clone(),
            birthday_store: self.birthday_store.clone(),
        }
    }
}

#[async_trait]
impl<AP: AuthenticationProvider, S: SubscriptionStore, CS: CalendarStore, BS: CalendarStore>
    ResourceService for PrincipalResourceService<AP, S, CS, BS>
{
    type PathComponents = (String,);
    type MemberType = CalendarSetResource;
    type Resource = PrincipalResource;
    type Error = Error;
    type Principal = User;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, calendar-access";

    async fn get_resource(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        let user = self
            .auth_provider
            .get_principal(principal)
            .await?
            .ok_or(crate::Error::NotFound)?;
        Ok(PrincipalResource {
            principal: user,
            home_set: &["calendar", "birthdays"],
        })
    }

    async fn get_members(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Vec<Self::MemberType>, Self::Error> {
        Ok(vec![
            CalendarSetResource {
                name: "calendar",
                principal: principal.to_owned(),
                read_only: false,
            },
            CalendarSetResource {
                name: "birthdays",
                principal: principal.to_owned(),
                read_only: true,
            },
        ])
    }

    fn axum_router<State: Send + Sync + Clone + 'static>(self) -> axum::Router<State> {
        Router::new()
            .nest(
                "/calendar",
                CalendarSetResourceService::new(
                    "calendar",
                    self.cal_store.clone(),
                    self.sub_store.clone(),
                )
                .axum_router(),
            )
            .nest(
                "/birthdays",
                CalendarSetResourceService::new(
                    "birthdays",
                    self.birthday_store.clone(),
                    self.sub_store.clone(),
                )
                .axum_router(),
            )
            .route_service("/", self.axum_service())
    }
}

impl<AP: AuthenticationProvider, S: SubscriptionStore, CS: CalendarStore, BS: CalendarStore>
    AxumMethods for PrincipalResourceService<AP, S, CS, BS>
{
}
