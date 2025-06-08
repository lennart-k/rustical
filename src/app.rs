use axum::Router;
use axum::extract::Request;
use axum::response::{Redirect, Response};
use axum::routing::{any, get};
use rustical_caldav::caldav_router;
use rustical_carddav::carddav_router;
use rustical_frontend::nextcloud_login::{NextcloudFlows, nextcloud_login_router};
use rustical_frontend::{FrontendConfig, frontend_router};
use rustical_oidc::OidcConfig;
use rustical_store::auth::AuthenticationProvider;
use rustical_store::{AddressbookStore, CalendarStore, SubscriptionStore};
use std::sync::Arc;
use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::TraceLayer;
use tower_sessions::MemoryStore;
use tracing::Span;

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
            any(async || Redirect::permanent("/caldav")),
        )
        .route(
            "/.well-known/carddav",
            any(async || Redirect::permanent("/carddav")),
        );

    let session_store = MemoryStore::default();
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
                    session_store.clone(),
                ),
            )
            .route("/", get(async || Redirect::to("/frontend")));
    }

    if nextcloud_login_config.enabled {
        router = router.nest(
            "/index.php/login/v2",
            nextcloud_login_router(
                nextcloud_flows_state,
                auth_provider.clone(),
                session_store.clone(),
            ),
        );
    }
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &Request| {
                tracing::debug_span!(
                    "http-request",
                    status_code = tracing::field::Empty,
                    otel.name =
                        tracing::field::display(format!("{} {}", request.method(), request.uri())),
                )
            })
            .on_request(|_req: &Request, _span: &Span| {})
            .on_response(|response: &Response, _latency: Duration, span: &Span| {
                span.record("status_code", tracing::field::display(response.status()));

                tracing::debug!("response generated")
            })
            .on_failure(
                |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                    tracing::error!("something went wrong")
                },
            ),
    )
}
