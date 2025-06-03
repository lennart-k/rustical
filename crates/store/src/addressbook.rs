use crate::synctoken::format_synctoken;
use chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Addressbook {
    pub id: String,
    pub principal: String,
    pub displayname: Option<String>,
    pub description: Option<String>,
    pub deleted_at: Option<NaiveDateTime>,
    pub synctoken: i64,
    pub push_topic: String,
}

impl Addressbook {
    pub fn format_synctoken(&self) -> String {
        format_synctoken(self.synctoken)
    }
}
