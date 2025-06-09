use crate::calendar::CalendarResourceService;
use crate::calendar::resource::CalendarResource;
use crate::calendar_set::CalendarSetResource;
use crate::{CalDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::Router;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_store::auth::User;
use rustical_store::{CalendarStore, SubscriptionStore};
use std::sync::Arc;

pub struct CalendarSetResourceService<C: CalendarStore, S: SubscriptionStore> {
    name: &'static str,
    cal_store: Arc<C>,
    sub_store: Arc<S>,
}

impl<C: CalendarStore, S: SubscriptionStore> Clone for CalendarSetResourceService<C, S> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            cal_store: self.cal_store.clone(),
            sub_store: self.sub_store.clone(),
        }
    }
}

impl<C: CalendarStore, S: SubscriptionStore> CalendarSetResourceService<C, S> {
    pub fn new(name: &'static str, cal_store: Arc<C>, sub_store: Arc<S>) -> Self {
        Self {
            name,
            cal_store,
            sub_store,
        }
    }
}

#[async_trait]
impl<C: CalendarStore, S: SubscriptionStore> ResourceService for CalendarSetResourceService<C, S> {
    type PathComponents = (String,);
    type MemberType = CalendarResource;
    type Resource = CalendarSetResource;
    type Error = Error;
    type Principal = User;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, extended-mkcol, calendar-access";
    const IS_COLLECTION: bool = true;

    async fn get_resource(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        Ok(CalendarSetResource {
            principal: principal.to_owned(),
            read_only: self.cal_store.is_read_only(),
            name: self.name,
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
                cal,
                read_only: self.cal_store.is_read_only(),
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
impl<C: CalendarStore, S: SubscriptionStore> AxumMethods for CalendarSetResourceService<C, S> {}
