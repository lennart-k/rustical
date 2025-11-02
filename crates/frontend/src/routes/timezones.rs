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

pub async fn route_timezones(method: Method) -> (HeaderMap, &'static str) {
    let mut headers = HeaderMap::new();
    headers.typed_insert(ContentType::json());
    headers.insert(
        "ETag",
        HeaderValue::from_static(vtimezones_rs::IANA_TZDB_VERSION),
    );
    headers.typed_insert(CacheControl::new().with_max_age(Duration::from_hours(2)));

    if method == Method::HEAD {
        return (headers, "");
    }
    (headers, VTIMEZONES_JSON.as_str())
}

#[cfg(test)]
#[tokio::test]
async fn test_vtimezones_json() -> () {
    // Since there's an unwrap make sure this doesn't fail
    assert!(!VTIMEZONES_JSON.as_str().is_empty());

    assert!(route_timezones(Method::HEAD).await.1.is_empty());
    assert!(!route_timezones(Method::GET).await.1.is_empty());
}
