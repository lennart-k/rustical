use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Calendar {
    pub principal: String,
    pub id: String,
    pub displayname: Option<String>,
    pub order: i64,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: Option<String>,
    pub deleted_at: Option<NaiveDateTime>,
    pub synctoken: i64,
}
