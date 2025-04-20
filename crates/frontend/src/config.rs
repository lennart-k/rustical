use openidconnect::{ClientId, ClientSecret, IssuerUrl, Scope};
use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

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

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FrontendConfig {
    #[serde(serialize_with = "hex::serde::serialize")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub secret_key: [u8; 64],
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub allow_password_login: bool,
}
