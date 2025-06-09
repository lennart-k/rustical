use axum::Router;
use axum::extract::Request;
use axum::response::Response;
use reqwest::StatusCode;
use rustical_caldav::caldav_router;
use rustical_carddav::carddav_router;
use rustical_frontend::nextcloud_login::{NextcloudFlows, nextcloud_login_router};
use rustical_frontend::{FrontendConfig, frontend_router};
use rustical_oidc::OidcConfig;
use rustical_store::auth::AuthenticationProvider;
use rustical_store::{AddressbookStore, CalendarStore, SubscriptionStore};
use std::sync::Arc;
use std::time::Duration;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tracing::Span;
use tracing::field::display;

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
) -> Router<()> {
    let mut router = Router::new()
        .merge(caldav_router(
            "/caldav",
            auth_provider.clone(),
            cal_store.clone(),
            addr_store.clone(),
            subscription_store.clone(),
        ))
        .merge(carddav_router(
            "/carddav",
            auth_provider.clone(),
            addr_store.clone(),
            subscription_store.clone(),
        ));

    let session_store = MemoryStore::default();
    if frontend_config.enabled {
        router = router.merge(frontend_router(
            "/frontend",
            auth_provider.clone(),
            cal_store.clone(),
            addr_store.clone(),
            frontend_config,
            oidc_config,
        ));
    }

    if nextcloud_login_config.enabled {
        router = router.nest(
            "/index.php/login/v2",
            nextcloud_login_router(nextcloud_flows_state, auth_provider.clone()),
        );
    }
    router
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(true)
                .with_same_site(SameSite::Strict)
                .with_expiry(Expiry::OnInactivity(
                    tower_sessions::cookie::time::Duration::hours(2),
                )),
        )
        .layer(CatchPanicLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request| {
                    tracing::info_span!(
                        "http-request",
                        status_code = tracing::field::Empty,
                        otel.name = tracing::field::display(format!(
                            "{} {}",
                            request.method(),
                            request.uri()
                        )),
                    )
                })
                .on_request(|req: &Request, span: &Span| {
                    span.record("method", display(req.method()));
                    span.record("path", display(req.uri()));
                })
                .on_response(|response: &Response, _latency: Duration, span: &Span| {
                    span.record("status_code", display(response.status()));
                    if response.status().is_server_error() {
                        tracing::error!("server error");
                    } else if response.status().is_client_error() {
                        if response.status() == StatusCode::UNAUTHORIZED {
                            // The iOS client always tries an unauthenticated request first so
                            // logging 401's as errors would clog up our logs
                            tracing::debug!("unauthorized");
                        } else {
                            tracing::error!("client error");
                        }
                    };
                })
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        tracing::error!("something went wrong")
                    },
                ),
        )
}
