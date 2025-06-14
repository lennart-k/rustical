use crate::config::NextcloudLoginConfig;
use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use axum::routing::options;
use headers::{HeaderMapExt, UserAgent};
use http::{HeaderValue, StatusCode};
use rustical_caldav::caldav_router;
use rustical_carddav::carddav_router;
use rustical_frontend::nextcloud_login::nextcloud_login_router;
use rustical_frontend::{FrontendConfig, frontend_router};
use rustical_oidc::OidcConfig;
use rustical_store::auth::AuthenticationProvider;
use rustical_store::{
    AddressbookStore, CalendarStore, CombinedCalendarStore, ContactBirthdayStore, SubscriptionStore,
};
use std::sync::Arc;
use std::time::Duration;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tracing::Span;
use tracing::field::display;

#[allow(clippy::too_many_arguments)]
pub fn make_app<AS: AddressbookStore, CS: CalendarStore, S: SubscriptionStore>(
    addr_store: Arc<AS>,
    cal_store: Arc<CS>,
    subscription_store: Arc<S>,
    auth_provider: Arc<impl AuthenticationProvider>,
    frontend_config: FrontendConfig,
    oidc_config: Option<OidcConfig>,
    nextcloud_login_config: NextcloudLoginConfig,
) -> Router<()> {
    let combined_cal_store = Arc::new(CombinedCalendarStore::new(
        cal_store.clone(),
        ContactBirthdayStore::new(addr_store.clone()).into(),
    ));

    let mut router = Router::new()
        .merge(caldav_router(
            "/caldav",
            auth_provider.clone(),
            combined_cal_store.clone(),
            subscription_store.clone(),
        ))
        .merge(carddav_router(
            "/carddav",
            auth_provider.clone(),
            addr_store.clone(),
            subscription_store.clone(),
        ));

    // GNOME Accounts needs to discover a WebDAV Files endpoint to complete the setup
    // It looks at / as well as /remote.php/dav (Nextcloud)
    // This is not nice but we offer this as a sacrificial route to ensure the CalDAV/CardDAV setup
    // works.
    // See:
    // https://github.com/GNOME/gnome-online-accounts/blob/master/src/goabackend/goadavclient.c
    // https://github.com/GNOME/gnome-online-accounts/blob/master/src/goabackend/goawebdavprovider.c
    router = router.route(
        "/remote.php/dav",
        options(async || {
            let mut resp = Response::builder().status(StatusCode::OK);
            resp.headers_mut()
                .unwrap()
                .insert("DAV", HeaderValue::from_static("1"));
            resp.body(Body::empty()).unwrap()
        }),
    );

    let session_store = MemoryStore::default();
    if frontend_config.enabled {
        router = router.merge(frontend_router(
            "/frontend",
            auth_provider.clone(),
            combined_cal_store.clone(),
            addr_store.clone(),
            frontend_config,
            oidc_config,
        ));
    }

    if nextcloud_login_config.enabled {
        router = router.nest(
            "/index.php/login/v2",
            nextcloud_login_router(auth_provider.clone()),
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
                        status = tracing::field::Empty,
                        otel.name = tracing::field::display(format!(
                            "{} {}",
                            request.method(),
                            request.uri()
                        )),
                        ua = tracing::field::Empty,
                    )
                })
                .on_request(|req: &Request, span: &Span| {
                    span.record("method", display(req.method()));
                    span.record("path", display(req.uri()));
                    if let Some(ua) = req.headers().typed_get::<UserAgent>() {
                        span.record("ua", display(ua));
                    }
                })
                .on_response(|response: &Response, _latency: Duration, span: &Span| {
                    span.record("status", display(response.status()));
                    if response.status().is_server_error() {
                        tracing::error!("server error");
                    } else if response.status().is_client_error() {
                        match response.status() {
                            StatusCode::UNAUTHORIZED => {
                                // The iOS client always tries an unauthenticated request first so
                                // logging 401's as errors would clog up our logs
                                tracing::debug!("unauthorized");
                            }
                            StatusCode::NOT_FOUND => {
                                tracing::warn!("client error");
                            }
                            _ => {
                                tracing::error!("client error");
                            }
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
