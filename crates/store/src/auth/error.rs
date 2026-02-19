use axum::{body::Body, response::IntoResponse};
use derive_more::Display;
use http::{HeaderValue, Response, StatusCode, header};

#[derive(Clone, Debug, Display)]
pub struct UnauthorizedError;

impl IntoResponse for UnauthorizedError {
    fn into_response(self) -> axum::response::Response {
        let mut resp = Response::builder().status(StatusCode::UNAUTHORIZED);
        resp.headers_mut().unwrap().insert(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static(r#"Basic realm="RustiCal", charset="UTF-8""#),
        );
        resp.body(Body::empty()).unwrap()
    }
}
