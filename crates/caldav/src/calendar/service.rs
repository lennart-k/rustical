use crate::calendar::methods::get::route_get;
use crate::calendar::methods::mkcalendar::route_mkcalendar;
use crate::calendar::methods::post::route_post;
use crate::calendar::methods::report::route_report_calendar;
use crate::calendar::resource::CalendarResource;
use crate::calendar_object::CalendarObjectResourceService;
use crate::calendar_object::resource::CalendarObjectResource;
use crate::{CalDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::Router;
use axum::extract::Request;
use axum::handler::Handler;
use axum::response::Response;
use futures_util::future::BoxFuture;
use rustical_dav::resource::{AxumMethods, ResourceService};
use rustical_store::auth::Principal;
use rustical_store::{CalendarStore, SubscriptionStore};
use std::convert::Infallible;
use std::sync::Arc;
use tower::Service;

pub struct CalendarResourceService<C: CalendarStore, S: SubscriptionStore> {
    pub(crate) cal_store: Arc<C>,
    pub(crate) sub_store: Arc<S>,
}

impl<C: CalendarStore, S: SubscriptionStore> Clone for CalendarResourceService<C, S> {
    fn clone(&self) -> Self {
        Self {
            cal_store: self.cal_store.clone(),
            sub_store: self.sub_store.clone(),
        }
    }
}

impl<C: CalendarStore, S: SubscriptionStore> CalendarResourceService<C, S> {
    pub fn new(cal_store: Arc<C>, sub_store: Arc<S>) -> Self {
        Self {
            cal_store,
            sub_store,
        }
    }
}

#[async_trait]
impl<C: CalendarStore, S: SubscriptionStore> ResourceService for CalendarResourceService<C, S> {
    type MemberType = CalendarObjectResource;
    type PathComponents = (String, String); // principal, calendar_id
    type Resource = CalendarResource;
    type Error = Error;
    type Principal = Principal;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, calendar-access, calendar-proxy, webdav-push";

    async fn get_resource(
        &self,
        (principal, cal_id): &Self::PathComponents,
        show_deleted: bool,
    ) -> Result<Self::Resource, Error> {
        let calendar = self
            .cal_store
            .get_calendar(principal, cal_id, show_deleted)
            .await?;
        Ok(CalendarResource {
            cal: calendar,
            read_only: self.cal_store.is_read_only(cal_id),
        })
    }

    async fn get_members(
        &self,
        (principal, cal_id): &Self::PathComponents,
    ) -> Result<Vec<Self::MemberType>, Self::Error> {
        Ok(self
            .cal_store
            .get_objects(principal, cal_id)
            .await?
            .into_iter()
            .map(|object| CalendarObjectResource {
                object,
                principal: principal.to_owned(),
            })
            .collect())
    }

    async fn save_resource(
        &self,
        (principal, cal_id): &Self::PathComponents,
        file: Self::Resource,
    ) -> Result<(), Self::Error> {
        self.cal_store
            .update_calendar(principal.to_owned(), cal_id.to_owned(), file.into())
            .await?;
        Ok(())
    }

    async fn delete_resource(
        &self,
        (principal, cal_id): &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        self.cal_store
            .delete_calendar(principal, cal_id, use_trashbin)
            .await?;
        Ok(())
    }

    fn axum_router<State: Send + Sync + Clone + 'static>(self) -> axum::Router<State> {
        Router::new()
            .nest(
                "/{object_id}",
                CalendarObjectResourceService::new(self.cal_store.clone()).axum_router(),
            )
            .route_service("/", self.axum_service())
    }
}

impl<C: CalendarStore, S: SubscriptionStore> AxumMethods for CalendarResourceService<C, S> {
    fn report() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_report_calendar::<C, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn get() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_get::<C, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn post() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_post::<C, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn mkcalendar() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>>
    {
        Some(|state, req| {
            let mut service = Handler::with_state(route_mkcalendar::<C, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn mkcol() -> Option<fn(Self, Request) -> BoxFuture<'static, Result<Response, Infallible>>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_mkcalendar::<C, S>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }
}
