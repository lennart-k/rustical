use actix_session::SessionInsertError;
use actix_web::{
    HttpResponse, ResponseError, body::BoxBody, error::UrlGenerationError, http::StatusCode,
};
use openidconnect::{ClaimsVerificationError, ConfigurationError, url::ParseError};

#[derive(Debug, thiserror::Error)]
pub enum OidcError {
    #[error("Cannot generate redirect url, something's not configured correctly")]
    OidcParseError(#[from] ParseError),

    #[error("Cannot generate redirect url, something's not configured correctly")]
    ActixUrlGenerationError(#[from] UrlGenerationError),

    #[error(transparent)]
    OidcConfigurationError(#[from] ConfigurationError),

    #[error(transparent)]
    OidcClaimsVerificationError(#[from] ClaimsVerificationError),

    #[error(transparent)]
    SessionInsertError(#[from] SessionInsertError),

    #[error(transparent)]
    StoreError(#[from] rustical_store::Error),

    #[error("{0}")]
    Other(&'static str),
}

impl ResponseError for OidcError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}
