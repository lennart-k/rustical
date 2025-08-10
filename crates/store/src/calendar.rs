use std::str::FromStr;

use crate::synctoken::format_synctoken;
use chrono::NaiveDateTime;
use rustical_ical::CalendarObjectType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub principal: String,
    pub id: String,
    pub displayname: Option<String>,
    pub order: i64,
    pub description: Option<String>,
    pub color: Option<String>,
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

    pub fn get_timezone(&self) -> Option<chrono_tz::Tz> {
        self.timezone_id
            .as_ref()
            .and_then(|tzid| chrono_tz::Tz::from_str(tzid).ok())
    }

    pub fn get_vtimezone(&self) -> Option<&'static str> {
        self.timezone_id
            .as_ref()
            .and_then(|tzid| vtimezones_rs::VTIMEZONES.get(tzid).cloned())
    }
}
