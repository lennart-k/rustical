use axum::Router;
use axum::response::Redirect;
use axum::routing::get;
use rustical_caldav::caldav_router;
use rustical_carddav::carddav_router;
use rustical_frontend::nextcloud_login::NextcloudFlows;
use rustical_frontend::{FrontendConfig, frontend_router};
use rustical_oidc::OidcConfig;
use rustical_store::auth::AuthenticationProvider;
use rustical_store::{AddressbookStore, CalendarStore, SubscriptionStore};
use std::sync::Arc;

use crate::config::NextcloudLoginConfig;

#[allow(clippy::too_many_arguments)]
pub fn make_app<AS: AddressbookStore, CS: CalendarStore, S: SubscriptionStore>(
    addr_store: Arc<AS>,
    cal_store: Arc<CS>,
    subscription_store: Arc<S>,
    auth_provider: Arc<impl AuthenticationProvider>,
    frontend_config: FrontendConfig,
    oidc_config: Option<OidcConfig>,
    nextcloud_login_config: NextcloudLoginConfig,
    nextcloud_flows_state: Arc<NextcloudFlows>,
) -> Router {
    let mut router = Router::new()
        .nest(
            "/caldav",
            caldav_router(
                "/caldav",
                auth_provider.clone(),
                cal_store.clone(),
                addr_store.clone(),
                subscription_store.clone(),
            ),
        )
        .nest(
            "/carddav",
            carddav_router(
                "/carddav",
                auth_provider.clone(),
                addr_store.clone(),
                subscription_store.clone(),
            ),
        )
        .route(
            "/.well-known/caldav",
            get(async || Redirect::permanent("/caldav")),
        )
        .route(
            "/.well-known/carddav",
            get(async || Redirect::permanent("/caldav")),
        );

    if frontend_config.enabled {
        router = router
            .nest(
                "/frontend",
                frontend_router(
                    auth_provider.clone(),
                    cal_store.clone(),
                    addr_store.clone(),
                    frontend_config,
                    oidc_config,
                ),
            )
            .route("/", get(async || Redirect::to("/frontend")));
    }

    router

    // if nextcloud_login_config.enabled {
    //     app = app.configure(|cfg| {
    //         configure_nextcloud_login(
    //             cfg,
    //             nextcloud_flows_state,
    //             auth_provider.clone(),
    //             frontend_config.secret_key,
    //         )
    //     });
    // }
}
