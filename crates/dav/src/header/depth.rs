use axum::{body::Body, response::IntoResponse};
use headers::Header;
use http::{HeaderName, HeaderValue};
use rustical_xml::{ValueDeserialize, ValueSerialize, XmlError};
use std::str::FromStr;
use thiserror::Error;

static DEPTH: HeaderName = HeaderName::from_static("depth");

#[derive(Error, Debug)]
#[error("Invalid Depth header")]
pub struct InvalidDepthHeader;

impl IntoResponse for InvalidDepthHeader {
    fn into_response(self) -> axum::response::Response {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::BAD_REQUEST)
            .body(Body::empty())
            .expect("this always works")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Depth {
    Zero,
    One,
    #[default]
    Infinity,
}

impl From<&Depth> for &'static str {
    fn from(value: &Depth) -> Self {
        match value {
            Depth::Zero => "0",
            Depth::One => "1",
            Depth::Infinity => "infinity",
        }
    }
}

impl ValueSerialize for Depth {
    fn serialize(&self) -> String {
        let string: &str = self.into();
        string.to_owned()
    }
}

impl ValueDeserialize for Depth {
    fn deserialize(val: &str) -> Result<Self, XmlError> {
        Self::from_str(val)
            .map_err(|_| XmlError::InvalidVariant("Invalid value for depth".to_owned()))
    }
}

impl FromStr for Depth {
    type Err = InvalidDepthHeader;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Zero),
            "1" => Ok(Self::One),
            "Infinity" | "infinity" => Ok(Self::Infinity),
            _ => Err(InvalidDepthHeader),
        }
    }
}

impl Header for Depth {
    fn name() -> &'static HeaderName {
        &DEPTH
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
    use super::Depth;
    use axum::{body::Body, extract::FromRequest};
    use axum_extra::TypedHeader;
    use http::Request;

    #[tokio::test]
    #[rstest::rstest]
    #[case("0", Depth::Zero)]
    #[case("1", Depth::One)]
    #[case("infinity", Depth::Infinity)]
    async fn test_depth_header(#[case] input: &str, #[case] header: Depth) {
        let request = Request::builder()
            .method("GET")
            .header("Depth", input)
            .body(Body::empty())
            .unwrap();
        let TypedHeader(depth) = TypedHeader::<Depth>::from_request(request, &())
            .await
            .unwrap();
        assert_eq!(depth, header);
    }

    #[tokio::test]
    async fn test_invalid_depth_header() {
        let request = Request::builder()
            .method("GET")
            .header("Depth", "asldkj")
            .body(Body::empty())
            .unwrap();
        assert!(
            TypedHeader::<Depth>::from_request(request, &())
                .await
                .is_err()
        );
    }
}
