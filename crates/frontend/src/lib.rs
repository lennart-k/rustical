use axum::{
    Extension, RequestExt, Router,
    body::Body,
    extract::{OriginalUri, Request},
    middleware::{self, Next},
    response::{Redirect, Response},
    routing::{get, post},
};
use headers::{ContentType, HeaderMapExt};
use http::{Method, StatusCode};
use rustical_oidc::{OidcConfig, OidcServiceConfig, route_get_oidc_callback, route_post_oidc};
use rustical_store::{
    AddressbookStore, CalendarStore,
    auth::{AuthenticationProvider, middleware::AuthenticationLayer},
};
use std::sync::Arc;
use url::Url;

mod assets;
mod config;
pub mod nextcloud_login;
mod oidc_user_store;
mod routes;

pub use config::FrontendConfig;
use oidc_user_store::OidcUserStore;

use crate::routes::{
    addressbook::{route_addressbook, route_addressbook_restore},
    app_token::{route_delete_app_token, route_post_app_token},
    calendar::{route_calendar, route_calendar_restore},
    login::{route_get_login, route_post_login, route_post_logout},
    user::{route_get_home, route_root, route_user_named},
};
#[cfg(not(feature = "dev"))]
use assets::{Assets, EmbedService};

pub fn frontend_router<AP: AuthenticationProvider, CS: CalendarStore, AS: AddressbookStore>(
    prefix: &'static str,
    auth_provider: Arc<AP>,
    cal_store: Arc<CS>,
    addr_store: Arc<AS>,
    frontend_config: FrontendConfig,
    oidc_config: Option<OidcConfig>,
) -> Router {
    let mut router = Router::new();
    router = router
        .route("/", get(route_root))
        .route("/user", get(route_get_home))
        .route("/user/{user}", get(route_user_named::<CS, AS, AP>))
        // App token management
        .route("/user/{user}/app_token", post(route_post_app_token::<AP>))
        .route(
            // POST because HTML5 forms don't support DELETE method
            "/user/{user}/app_token/{id}/delete",
            post(route_delete_app_token::<AP>),
        )
        // Calendar
        .route(
            "/user/{user}/calendar/{calendar}",
            get(route_calendar::<CS>),
        )
        .route(
            "/user/{user}/calendar/{calendar}/restore",
            post(route_calendar_restore::<CS>),
        )
        // Addressbook
        .route(
            "/user/{user}/addressbook/{addressbook}",
            get(route_addressbook::<AS>),
        )
        .route(
            "/user/{user}/addressbook/{addressbook}/restore",
            post(route_addressbook_restore::<AS>),
        )
        .route("/login", get(route_get_login).post(route_post_login::<AP>))
        .route("/logout", post(route_post_logout));

    #[cfg(not(feature = "dev"))]
    let mut router = router.route_service("/assets/{*file}", EmbedService::<Assets>::default());
    #[cfg(feature = "dev")]
    let mut router = router.nest_service(
        "/assets",
        tower_http::services::ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/public/assets")),
    );

    if let Some(oidc_config) = oidc_config.clone() {
        router = router
            .route("/login/oidc", post(route_post_oidc))
            .route(
                "/login/oidc/callback",
                get(route_get_oidc_callback::<OidcUserStore<AP>>),
            )
            .layer(Extension(OidcUserStore(auth_provider.clone())))
            .layer(Extension(OidcServiceConfig {
                default_redirect_path: "/frontend/user",
                session_key_user_id: "user",
            }))
            .layer(Extension(oidc_config));
    }

    router = router
        .layer(AuthenticationLayer::new(auth_provider.clone()))
        .layer(Extension(auth_provider.clone()))
        .layer(Extension(cal_store.clone()))
        .layer(Extension(addr_store.clone()))
        .layer(Extension(frontend_config.clone()))
        .layer(Extension(oidc_config.clone()))
        .layer(middleware::from_fn(unauthorized_handler));

    Router::new()
        .nest(prefix, router)
        .route("/", get(async || Redirect::to(prefix)))
}

async fn unauthorized_handler(mut request: Request, next: Next) -> Response {
    let meth = request.method().clone();
    let OriginalUri(uri) = request.extract_parts().await.unwrap();
    let resp = next.run(request).await;
    if resp.status() == StatusCode::UNAUTHORIZED {
        // This is a dumb hack since parsed Urls cannot be relative
        let mut login_url: Url = "http://github.com/frontend/login".parse().unwrap();
        if meth == Method::GET {
            login_url
                .query_pairs_mut()
                .append_pair("redirect_uri", uri.path());
        }
        let path = login_url.path();
        let query = login_url
            .query()
            .map(|query| format!("?{query}"))
            .unwrap_or_default();
        let login_url = format!("{path}{query}");
        let mut resp = Response::builder().status(StatusCode::UNAUTHORIZED);
        let hdrs = resp.headers_mut().unwrap();
        hdrs.typed_insert(ContentType::html());
        return resp
            .body(Body::new(format!(
                r#"<!Doctype html>
                <html>
                    <head>
                        <meta http-equiv="refresh" content="1; url={login_url}" />
                    </head>
                    <body>
                        Unauthorized, redirecting to <a href="{login_url}">login page</a>
                    </body>
                <html>
            "#,
            )))
            .unwrap();
    }
    resp
}
