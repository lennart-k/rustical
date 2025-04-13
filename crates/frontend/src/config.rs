use openidconnect::{ClientId, ClientSecret, IssuerUrl, Scope};
use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    true
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OidcConfig {
    pub name: String,
    pub issuer: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: Option<ClientSecret>,
    pub scopes: Vec<Scope>,
    pub allow_sign_up: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct FrontendConfig {
    #[serde(serialize_with = "hex::serde::serialize")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub secret_key: [u8; 64],
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub oidc: Option<OidcConfig>,
}
