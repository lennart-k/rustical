use axum::{
    body::Body,
    extract::{FromRequestParts, OptionalFromRequestParts},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use derive_more::Display;
use http::{HeaderValue, StatusCode, header};
use rustical_xml::ValueSerialize;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, fmt::Display};

use crate::Secret;

/// https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.3
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, clap::ValueEnum)]
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

impl TryFrom<&str> for PrincipalType {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "INDIVIDUAL" => Self::Individual,
            "GROUP" => Self::Group,
            "RESOURCE" => Self::Resource,
            "ROOM" => Self::Room,
            "UNKNOWN" => Self::Unknown,
            _ => {
                return Err(crate::Error::InvalidPrincipalType(
                    "Invalid principal type".to_owned(),
                ));
            }
        })
    }
}

impl PrincipalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrincipalType::Individual => "INDIVIDUAL",
            PrincipalType::Group => "GROUP",
            PrincipalType::Resource => "RESOURCE",
            PrincipalType::Room => "ROOM",
            PrincipalType::Unknown => "UNKNOWN",
        }
    }
}

impl Display for PrincipalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ValueSerialize for PrincipalType {
    fn serialize(&self) -> String {
        self.to_string()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppToken {
    pub id: String,
    pub name: String,
    pub token: Secret<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Principal {
    pub id: String,
    pub displayname: Option<String>,
    #[serde(default)]
    pub principal_type: PrincipalType,
    pub password: Option<Secret<String>>,
    #[serde(default)]
    pub memberships: Vec<String>,
}

impl Principal {
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

    pub fn memberships_without_self(&self) -> Vec<&str> {
        self.memberships.iter().map(String::as_str).collect()
    }
}

impl rustical_dav::Principal for Principal {
    fn get_id(&self) -> &str {
        &self.id
    }
}

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

impl<S: Send + Sync + Clone> FromRequestParts<S> for Principal {
    type Rejection = UnauthorizedError;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Self>()
            .cloned()
            .ok_or(UnauthorizedError)
    }
}

impl<S: Send + Sync + Clone> OptionalFromRequestParts<S> for Principal {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        _state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        Ok(parts.extensions.get::<Self>().cloned())
    }
}
