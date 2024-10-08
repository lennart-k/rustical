use rustical_frontend::FrontendConfig;
use rustical_store::auth::StaticUserStoreConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SqliteCalendarStoreConfig {
    pub db_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum CalendarStoreConfig {
    Sqlite(SqliteCalendarStoreConfig),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum AuthConfig {
    Static(StaticUserStoreConfig),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TracingConfig {
    pub opentelemetry: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub calendar_store: CalendarStoreConfig,
    pub auth: AuthConfig,
    pub http: HttpConfig,
    pub frontend: FrontendConfig,
    pub tracing: TracingConfig,
}
