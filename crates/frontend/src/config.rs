use rand::RngCore;
use serde::{Deserialize, Serialize};

pub fn generate_frontend_secret() -> [u8; 64] {
    let mut rng = rand::thread_rng();

    let mut secret = [0u8; 64];
    rng.fill_bytes(&mut secret);
    secret
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FrontendConfig {
    #[serde(serialize_with = "hex::serde::serialize")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    #[serde(default = "generate_frontend_secret")]
    pub secret_key: [u8; 64],
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub allow_password_login: bool,
}
