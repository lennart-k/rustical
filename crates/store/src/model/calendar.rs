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

impl Calendar {
    pub fn format_synctoken(&self) -> String {
        format_synctoken(self.synctoken)
    }
}

const SYNC_NAMESPACE: &str = "github.com/lennart-k/rustical/ns/";

pub fn format_synctoken(synctoken: i64) -> String {
    format!("{}{}", SYNC_NAMESPACE, synctoken)
}

pub fn parse_synctoken(synctoken: &str) -> Option<i64> {
    if !synctoken.starts_with(SYNC_NAMESPACE) {
        return None;
    }
    let (_, synctoken) = synctoken.split_at(SYNC_NAMESPACE.len());
    synctoken.parse::<i64>().ok()
}
