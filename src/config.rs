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
pub struct Config {
    pub calendar_store: CalendarStoreConfig,
}
