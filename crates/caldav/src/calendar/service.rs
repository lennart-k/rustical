use crate::calendar::methods::get::route_get;
use crate::calendar::methods::import::route_import;
use crate::calendar::methods::mkcalendar::route_mkcalendar;
use crate::calendar::methods::post::route_post;
use crate::calendar::methods::report::route_report_calendar;
use crate::calendar::resource::{CalendarResource, objects_read_only};
use crate::calendar_object::CalendarObjectResourceService;
use crate::calendar_object::resource::CalendarObjectResource;
use crate::{CalDavConfig, CalDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::handler::Handler;
use rustical_dav::resource::{AxumMethods, MethodFunction, ResourceService};
use rustical_dav_push::DavPushStore;
use rustical_store::auth::Principal;
use rustical_store::{CalendarStore, ShareStore, ShareViewUpdate, access, shared};
use std::sync::Arc;
use tower::Service;

pub struct CalendarResourceService<C: CalendarStore, DP: DavPushStore> {
    pub(crate) cal_store: Arc<C>,
    pub(crate) share_store: Arc<dyn ShareStore>,
    pub(crate) dav_push_store: Arc<DP>,
    pub(crate) config: Arc<CalDavConfig>,
}

impl<C: CalendarStore, DP: DavPushStore> Clone for CalendarResourceService<C, DP> {
    fn clone(&self) -> Self {
        Self {
            cal_store: self.cal_store.clone(),
            share_store: self.share_store.clone(),
            dav_push_store: self.dav_push_store.clone(),
            config: self.config.clone(),
        }
    }
}

impl<C: CalendarStore, DP: DavPushStore> CalendarResourceService<C, DP> {
    pub fn new(
        cal_store: Arc<C>,
        share_store: Arc<dyn ShareStore>,
        dav_push_store: Arc<DP>,
        config: Arc<CalDavConfig>,
    ) -> Self {
        Self {
            cal_store,
            share_store,
            dav_push_store,
            config,
        }
    }

    /// The calendar at the path as a resource
    pub(crate) async fn resource(
        &self,
        principal: &str,
        cal_id: &str,
        show_deleted: bool,
    ) -> Result<CalendarResource, Error> {
        let (resolved, calendar) = access::resolve_calendar(
            self.share_store.as_ref(),
            self.cal_store.as_ref(),
            principal,
            cal_id,
            show_deleted,
        )
        .await?;
        let vapid_pubkey = self.dav_push_store.get_vapid_pubkey_b64().await?.clone();
        Ok(CalendarResource {
            read_only: objects_read_only(self.cal_store.as_ref(), &calendar),
            cal: calendar,
            vapid_pubkey,
            home: principal.to_owned(),
            name: cal_id.to_owned(),
            resolved,
        })
    }
}

#[async_trait]
impl<C: CalendarStore, DP: DavPushStore> ResourceService for CalendarResourceService<C, DP> {
    type MemberType = CalendarObjectResource;
    type PathComponents = (String, String); // principal, calendar_id
    type Resource = CalendarResource;
    type Error = Error;
    type Principal = Principal;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, calendar-access, webdav-push";

    async fn get_resource(
        &self,
        (principal, cal_id): &Self::PathComponents,
        show_deleted: bool,
    ) -> Result<Self::Resource, Error> {
        self.resource(principal, cal_id, show_deleted).await
    }

    async fn get_members(
        &self,
        (principal, cal_id): &Self::PathComponents,
    ) -> Result<Vec<Self::MemberType>, Self::Error> {
        let calendar = self.resource(principal, cal_id, true).await?;
        Ok(shared::objects(self.cal_store.as_ref(), &calendar.resolved)
            .await?
            .into_iter()
            .map(|(object_id, object)| {
                let level = shared::level(&calendar.resolved, &object);
                CalendarObjectResource::new(
                    object,
                    object_id,
                    principal,
                    &calendar.resolved,
                    calendar.read_only,
                    level,
                )
            })
            .collect())
    }

    async fn save_resource(
        &self,
        (principal, cal_id): &Self::PathComponents,
        file: Self::Resource,
    ) -> Result<(), Self::Error> {
        if file.resolved.is_own() {
            self.cal_store
                .update_calendar(principal, cal_id, file.into())
                .await?;
            return Ok(());
        }
        // A sharee changes only its own view; only what changed is written
        let before = self.resource(principal, cal_id, false).await?;
        let (new, old) = (&file.cal.meta, &before.cal.meta);
        let update = ShareViewUpdate {
            displayname: (new.displayname != old.displayname).then(|| new.displayname.clone()),
            color: (new.color != old.color).then(|| new.color.clone()),
            order: (new.order != old.order).then_some(Some(new.order)),
        };
        if !update.is_empty() {
            self.share_store
                .update_calendar_share_view(principal, cal_id, update)
                .await?;
        }
        Ok(())
    }

    async fn delete_resource(
        &self,
        (principal, cal_id): &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        let calendar = self.resource(principal, cal_id, true).await?;
        if calendar.resolved.is_own() {
            self.cal_store
                .delete_calendar(principal, cal_id, use_trashbin)
                .await?;
        } else {
            // Deleting a share only ends it for this home
            self.share_store
                .unshare_calendar(calendar.resolved.owner(), calendar.resolved.id(), principal)
                .await?;
        }
        Ok(())
    }

    fn axum_router<State: Send + Sync + Clone + 'static>(self) -> axum::Router<State> {
        Router::new()
            .nest(
                "/{object_id}",
                CalendarObjectResourceService::new(
                    self.cal_store.clone(),
                    self.share_store.clone(),
                    self.config.clone(),
                )
                .axum_router(),
            )
            .route_service("/", self.axum_service())
    }
}

impl<C: CalendarStore, DP: DavPushStore> AxumMethods for CalendarResourceService<C, DP> {
    fn report() -> Option<MethodFunction<Self>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_report_calendar::<C, DP>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn get() -> Option<MethodFunction<Self>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_get::<C, DP>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn post() -> Option<MethodFunction<Self>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_post::<C, DP>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn import() -> Option<MethodFunction<Self>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_import::<C, DP>, state);
            Box::pin(Service::call(&mut service, req))
        })
    }

    fn mkcalendar() -> Option<MethodFunction<Self>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_mkcalendar::<C, DP>, state);
            Box::pin(Service::<Request<Body>>::call(&mut service, req))
        })
    }

    fn mkcol() -> Option<MethodFunction<Self>> {
        Some(|state, req| {
            let mut service = Handler::with_state(route_mkcalendar::<C, DP>, state);
            Box::pin(Service::<Request<Body>>::call(&mut service, req))
        })
    }
}
