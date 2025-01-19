use serde::{Deserialize, Serialize};

fn default_enabled() -> bool {
    true
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct FrontendConfig {
    #[serde(serialize_with = "hex::serde::serialize")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub secret_key: [u8; 64],
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}
