use axum::response::{IntoResponse, Response};
use headers::{CacheControl, ContentType, HeaderMapExt};
use http::{HeaderMap, HeaderValue, Method};
use itertools::Itertools;
use std::{sync::LazyLock, time::Duration};

static VTIMEZONES_JSON: LazyLock<String> = LazyLock::new(|| {
    serde_json::to_string(
        &vtimezones_rs::VTIMEZONES
            .keys()
            .sorted()
            .collect::<Vec<_>>(),
    )
    .unwrap()
});

pub async fn route_timezones(method: Method) -> Response {
    let mut headers = HeaderMap::new();
    headers.typed_insert(ContentType::json());
    headers.insert(
        "ETag",
        HeaderValue::from_static(vtimezones_rs::IANA_TZDB_VERSION),
    );
    headers.typed_insert(CacheControl::new().with_max_age(Duration::from_hours(2)));

    if method == Method::HEAD {
        return headers.into_response();
    }
    (headers, VTIMEZONES_JSON.as_str()).into_response()
}
