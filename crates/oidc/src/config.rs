use crate::error::OidcError;
use openidconnect::{
    AdditionalClaims, Audience, ClientId, ClientSecret, GenderClaim, IssuerUrl, Scope,
    UserInfoClaims,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum UserIdClaim {
    // The correct option
    Sub,
    // The more ergonomic option if you know what you're doing
    #[default]
    PreferredUsername,
    // The hopefully unique option
    Email,
}

impl UserIdClaim {
    pub fn extract_user_id<AC: AdditionalClaims, GC: GenderClaim>(
        &self,
        claims: &UserInfoClaims<AC, GC>,
    ) -> Result<String, OidcError> {
        Ok(match self {
            Self::Sub => claims.subject().to_string(),
            Self::PreferredUsername => claims
                .preferred_username()
                .ok_or(OidcError::Other("Missing preferred_username claim"))?
                .to_string(),
            Self::Email => claims
                .email()
                .ok_or(OidcError::Other("Missing email claim"))?
                .to_string(),
        })
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct OidcConfig {
    pub name: String,
    pub issuer: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: Option<ClientSecret>,
    pub scopes: Vec<Scope>,
    #[serde(default)]
    pub allow_sign_up: bool,
    pub require_group: Option<String>,
    #[serde(default)]
    pub claim_userid: UserIdClaim,
    #[serde(default)]
    pub additional_audiences: Vec<Audience>,
    #[serde(default)]
    pub assign_memberships: HashMap<String, Vec<String>>,
}
