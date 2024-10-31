use rustical_frontend::FrontendConfig;
use rustical_store::auth::StaticUserStoreConfig;
use serde::{Deserialize, Serialize};

fn http_default_host() -> String {
    "0.0.0.0".to_owned()
}
fn http_default_port() -> u16 {
    4000
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpConfig {
    #[serde(default = "http_default_host")]
    pub host: String,
    #[serde(default = "http_default_port")]
    pub port: u16,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            host: http_default_host(),
            port: http_default_port(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SqliteDataStoreConfig {
    pub db_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum DataStoreConfig {
    Sqlite(SqliteDataStoreConfig),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum AuthConfig {
    Static(StaticUserStoreConfig),
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct TracingConfig {
    pub opentelemetry: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub data_store: DataStoreConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub http: HttpConfig,
    pub frontend: FrontendConfig,
    #[serde(default)]
    pub tracing: TracingConfig,
}
