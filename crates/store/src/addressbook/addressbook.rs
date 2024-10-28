use crate::synctoken::format_synctoken;
use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Addressbook {
    pub id: String,
    pub principal: String,
    pub displayname: Option<String>,
    pub description: Option<String>,
    pub deleted_at: Option<NaiveDateTime>,
    pub synctoken: i64,
}

impl Addressbook {
    pub fn format_synctoken(&self) -> String {
        format_synctoken(self.synctoken)
    }
}
