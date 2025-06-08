use openidconnect::{ClientId, ClientSecret, IssuerUrl, Scope};
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum UserIdClaim {
    // The correct option
    Sub,
    // The more ergonomic option if you know what you're doing
    #[default]
    PreferredUsername,
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
}
