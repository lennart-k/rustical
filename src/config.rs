use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonCalendarStoreConfig {
    pub db_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum CalendarStoreConfig {
    Json(JsonCalendarStoreConfig),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub calendar_store: CalendarStoreConfig,
}
