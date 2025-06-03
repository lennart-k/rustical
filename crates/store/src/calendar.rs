use crate::synctoken::format_synctoken;
use chrono::NaiveDateTime;
use rustical_ical::CalendarObjectType;
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Calendar {
    pub principal: String,
    pub id: String,
    pub displayname: Option<String>,
    pub order: i64,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: Option<String>,
    pub timezone_id: Option<String>,
    pub deleted_at: Option<NaiveDateTime>,
    pub synctoken: i64,
    pub subscription_url: Option<String>,
    pub push_topic: String,
    pub components: Vec<CalendarObjectType>,
}

impl Calendar {
    pub fn format_synctoken(&self) -> String {
        format_synctoken(self.synctoken)
    }
}
