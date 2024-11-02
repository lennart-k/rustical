use rustical_frontend::FrontendConfig;
use rustical_store::auth::StaticUserStoreConfig;
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
#[serde(tag = "backend", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum DataStoreConfig {
    Sqlite(SqliteDataStoreConfig),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum AuthConfig {
    Static(StaticUserStoreConfig),
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields, default)]
pub struct TracingConfig {
    pub opentelemetry: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub data_store: DataStoreConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub http: HttpConfig,
    pub frontend: FrontendConfig,
    #[serde(default)]
    pub tracing: TracingConfig,
}
