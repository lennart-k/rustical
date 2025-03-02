use actix_web::{
    body::BoxBody,
    http::{header, StatusCode},
    FromRequest, HttpMessage, HttpResponse, ResponseError,
};
use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts},
    http::HeaderName,
    response::{IntoResponse, IntoResponseParts},
};
use base64::Engine;
use chrono::{DateTime, Utc};
use derive_more::Display;
use rustical_xml::ValueSerialize;
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

use super::AuthenticationProvider;

/// https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.3
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PrincipalType {
    #[default]
    Individual,
    Group,
    Resource,
    Room,
    Unknown,
    // TODO: X-Name, IANA-token
}

impl ValueSerialize for PrincipalType {
    fn serialize(&self) -> String {
        match self {
            PrincipalType::Individual => "INDIVIDUAL",
            PrincipalType::Group => "GROUP",
            PrincipalType::Resource => "RESOURCE",
            PrincipalType::Room => "ROOM",
            PrincipalType::Unknown => "UNKNOWN",
        }
        .to_owned()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppToken {
    pub name: String,
    pub token: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
// TODO: Rename this to Principal
pub struct User {
    pub id: String,
    pub displayname: Option<String>,
    #[serde(default)]
    pub principal_type: PrincipalType,
    pub password: Option<String>,
    #[serde(default)]
    pub app_tokens: Vec<AppToken>,
    #[serde(default)]
    pub memberships: Vec<String>,
}

impl User {
    /// Returns true if the user is either
    /// - the principal itself
    /// - has full access to the prinicpal (is member)
    pub fn is_principal(&self, principal: &str) -> bool {
        if self.id == principal {
            return true;
        }
        self.memberships
            .iter()
            .any(|membership| membership == principal)
    }

    /// Returns all principals the user implements
    pub fn memberships(&self) -> Vec<&str> {
        let mut memberships: Vec<_> = self.memberships.iter().map(String::as_ref).collect();
        memberships.push(self.id.as_str());
        memberships
    }
}

#[derive(Clone, Debug, Display)]
pub struct UnauthorizedError;

impl IntoResponse for UnauthorizedError {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::UNAUTHORIZED, "Unauthorized").into_response()
    }
}

impl ResponseError for UnauthorizedError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        StatusCode::UNAUTHORIZED
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::build(StatusCode::UNAUTHORIZED)
            .insert_header((
                header::WWW_AUTHENTICATE,
                r#"Basic realm="RustiCal", charset="UTF-8""#,
            ))
            .finish()
    }
}

pub trait ToAuthenticationProvider {
    fn auth_provider(&self) -> &impl AuthenticationProvider;
}

impl FromRequest for User {
    type Error = UnauthorizedError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(
            req.extensions()
                .get::<Self>()
                .cloned()
                .ok_or(UnauthorizedError),
        )
    }
}

// Just a MVP with code stolen from https://github.com/Owez/axum-auth/blob/master/src/auth_basic.rs
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync + ToAuthenticationProvider,
{
    type Rejection = UnauthorizedError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Some(Ok(auth_header)) = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .map(|val| val.to_str())
        {
            if let Some(("Basic", contents)) = auth_header.split_once(' ') {
                if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(contents) {
                    if let Ok(decoded) = String::from_utf8(decoded) {
                        // Return depending on if password is present
                        if let Some((id, password)) = decoded.split_once(':') {
                            if let Ok(Some(user)) = state
                                .auth_provider()
                                .validate_user_token(id, password)
                                .await
                            {
                                return Ok(user);
                            }
                        }
                    }
                }
            }
        }
        Err(UnauthorizedError)
    }
}
