use rustical_auth::{AuthProvider, HtpasswdAuthConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TomlCalendarStoreConfig {
    pub db_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum CalendarStoreConfig {
    Toml(TomlCalendarStoreConfig),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum AuthConfig {
    Htpasswd(HtpasswdAuthConfig),
    None,
}

impl From<AuthConfig> for AuthProvider {
    fn from(value: AuthConfig) -> Self {
        match value {
            AuthConfig::Htpasswd(config) => {
                Self::Htpasswd(rustical_auth::htpasswd::HtpasswdAuth { config })
            }
            AuthConfig::None => Self::None(rustical_auth::none::NoneAuth),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub calendar_store: CalendarStoreConfig,
    pub auth: AuthConfig,
}
