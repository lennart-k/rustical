use axum::{body::Body, response::IntoResponse};
use derive_more::{From, Into};
use headers::Header;
use http::{HeaderName, HeaderValue};
use std::str::FromStr;
use thiserror::Error;

static OVERWRITE: HeaderName = HeaderName::from_static("overwrite");

#[derive(Error, Debug)]
#[error("Invalid Overwrite header")]
pub struct InvalidOverwriteHeader;

impl IntoResponse for InvalidOverwriteHeader {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::BAD_REQUEST)
            .body(Body::new("Invalid Overwrite header".to_string()))
            .expect("this always works")
    }
}

#[derive(Debug, PartialEq, Eq, From, Into)]
pub struct Overwrite(pub bool);

impl Default for Overwrite {
    fn default() -> Self {
        Self(true)
    }
}

impl From<&Overwrite> for &'static str {
    fn from(value: &Overwrite) -> Self {
        if value.0 { "T" } else { "F" }
    }
}

impl FromStr for Overwrite {
    type Err = InvalidOverwriteHeader;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "T" => Ok(Self(true)),
            "F" => Ok(Self(false)),
            _ => Err(InvalidOverwriteHeader),
        }
    }
}

impl Header for Overwrite {
    fn name() -> &'static HeaderName {
        &OVERWRITE
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(HeaderValue::from_static(self.into())));
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let Some(val) = values.next() else {
            return Err(headers::Error::invalid());
        };
        if values.next().is_some() {
            return Err(headers::Error::invalid());
        }
        let val = val.to_str().map_err(|_| headers::Error::invalid())?;
        Self::from_str(val).map_err(|_| headers::Error::invalid())
    }
}

#[cfg(test)]
mod tests {
    use super::Overwrite;
    use axum::{body::Body, extract::FromRequest};
    use axum_extra::TypedHeader;
    use http::Request;

    #[tokio::test]
    #[rstest::rstest]
    #[case("T", Overwrite(true))]
    #[case("F", Overwrite(false))]
    async fn test_overwrite_header(#[case] input: &str, #[case] header: Overwrite) {
        let request = Request::builder()
            .method("GET")
            .header("Overwrite", input)
            .body(Body::empty())
            .unwrap();
        let TypedHeader(depth) = TypedHeader::<Overwrite>::from_request(request, &())
            .await
            .unwrap();
        assert_eq!(depth, header);
    }

    #[tokio::test]
    async fn test_invalid_overwrite_header() {
        let request = Request::builder()
            .method("GET")
            .header("Overwrite", "asldkj")
            .body(Body::empty())
            .unwrap();
        assert!(
            TypedHeader::<Overwrite>::from_request(request, &())
                .await
                .is_err()
        );
    }
}
