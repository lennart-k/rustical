use axum::http::StatusCode;
use axum::response::IntoResponse;
use openidconnect::{ClaimsVerificationError, ConfigurationError, url::ParseError};

#[derive(Debug, thiserror::Error)]
pub enum OidcError {
    #[error("Cannot generate redirect url, something's not configured correctly")]
    OidcParseError(#[from] ParseError),

    #[error("Error fetching user info: {0}")]
    UserInfo(String),

    #[error(transparent)]
    OidcConfigurationError(#[from] ConfigurationError),

    #[error(transparent)]
    OidcClaimsVerificationError(#[from] ClaimsVerificationError),

    #[error(transparent)]
    SessionError(#[from] tower_sessions::session::Error),

    #[error("{0}")]
    Other(&'static str),
}

impl IntoResponse for OidcError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
