use rustical_frontend::FrontendConfig;
use rustical_oidc::OidcConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_owned(),
            port: 4000,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteDataStoreConfig {
    pub db_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum DataStoreConfig {
    Sqlite(SqliteDataStoreConfig),
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct TracingConfig {
    pub opentelemetry: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct DavPushConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    // Allowed Push servers, accepts any by default
    // Specify as URL origins
    pub allowed_push_servers: Option<Vec<String>>,
}

impl Default for DavPushConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_push_servers: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct NextcloudLoginConfig {
    pub enabled: bool,
}

impl Default for NextcloudLoginConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub data_store: DataStoreConfig,
    #[serde(default)]
    pub http: HttpConfig,
    pub frontend: FrontendConfig,
    #[serde(default)]
    pub oidc: Option<OidcConfig>,
    #[serde(default)]
    pub tracing: TracingConfig,
    #[serde(default)]
    pub dav_push: DavPushConfig,
    #[serde(default)]
    pub nextcloud_login: NextcloudLoginConfig,
}
