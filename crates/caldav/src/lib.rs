use axum::{Extension, Router};
use derive_more::Constructor;
use principal::PrincipalResourceService;
use rustical_dav::resource::{PrincipalUri, ResourceService};
use rustical_dav::resources::RootResourceService;
use rustical_store::auth::middleware::AuthenticationLayer;
use rustical_store::auth::{AuthenticationProvider, User};
use rustical_store::{AddressbookStore, CalendarStore, ContactBirthdayStore, SubscriptionStore};
use std::sync::Arc;

pub mod calendar;
pub mod calendar_object;
pub mod calendar_set;
pub mod error;
pub mod principal;
// mod subscription;

pub use error::Error;

use crate::calendar::resource::CalendarResourceService;
use crate::calendar_object::resource::CalendarObjectResourceService;
use crate::calendar_set::CalendarSetResourceService;

#[derive(Debug, Clone, Constructor)]
pub struct CalDavPrincipalUri(&'static str);

impl PrincipalUri for CalDavPrincipalUri {
    fn principal_uri(&self, principal: &str) -> String {
        format!("{}/principal/{}", self.0, principal)
    }
}

pub fn caldav_router<
    AP: AuthenticationProvider,
    AS: AddressbookStore,
    C: CalendarStore,
    S: SubscriptionStore,
>(
    prefix: &'static str,
    auth_provider: Arc<AP>,
    store: Arc<C>,
    addr_store: Arc<AS>,
    subscription_store: Arc<S>,
) -> Router {
    let birthday_store = Arc::new(ContactBirthdayStore::new(addr_store));
    let principal_service = PrincipalResourceService {
        auth_provider: auth_provider.clone(),
        sub_store: subscription_store.clone(),
        birthday_store: birthday_store.clone(),
        cal_store: store.clone(),
    };

    Router::new()
        .route_service(
            "/",
            RootResourceService::<_, User, CalDavPrincipalUri>::new(principal_service.clone())
                .axum_service(),
        )
        .route_service("/principal/{principal}", principal_service.axum_service())
        .route_service(
            "/principal/{principal}/calendar",
            CalendarSetResourceService::new("calendar", store.clone(), subscription_store.clone())
                .axum_service(),
        )
        .route_service(
            "/principal/{principal}/calendar/{calendar_id}",
            CalendarResourceService::new(store.clone(), subscription_store.clone()).axum_service(),
        )
        .route_service(
            "/principal/{principal}/calendar/{calendar_id}/{object_id}",
            CalendarObjectResourceService::new(store.clone()).axum_service(),
        )
        .route_service(
            "/principal/{principal}/birthdays",
            CalendarSetResourceService::new(
                "birthdays",
                birthday_store.clone(),
                subscription_store.clone(),
            )
            .axum_service(),
        )
        .route_service(
            "/principal/{principal}/birthdays/{calendar_id}",
            CalendarResourceService::new(birthday_store.clone(), subscription_store.clone())
                .axum_service(),
        )
        .route_service(
            "/principal/{principal}/birthdays/{calendar_id}/{object_id}",
            CalendarObjectResourceService::new(birthday_store.clone()).axum_service(),
        )
        .layer(AuthenticationLayer::new(auth_provider))
        .layer(Extension(CalDavPrincipalUri(prefix)))
}
