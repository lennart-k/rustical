use std::io::BufReader;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ical::generator::{Emitter, IcalCalendar};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct Event {
    uid: String,
    cal: IcalCalendar,
}

// Custom implementation for Event (de)serialization
impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Inner {
            uid: String,
            ics: String,
        }
        let Inner { uid, ics } = Inner::deserialize(deserializer)?;
        Self::from_ics(uid, ics).map_err(serde::de::Error::custom)
    }
}
impl Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Inner {
            uid: String,
            ics: String,
        }
        Inner::serialize(
            &Inner {
                uid: self.get_uid().to_string(),
                ics: self.get_ics().to_string(),
            },
            serializer,
        )
    }
}

impl Event {
    pub fn from_ics(uid: String, ics: String) -> Result<Self> {
        let mut parser = ical::IcalParser::new(BufReader::new(ics.as_bytes()));
        let cal = parser.next().ok_or(anyhow!("no calendar :("))??;
        if parser.next().is_some() {
            return Err(anyhow!("multiple calendars!"));
        }
        if cal.events.len() == 2 {
            return Err(anyhow!("multiple events"));
        }
        Ok(Self { uid, cal })
    }
    pub fn get_uid(&self) -> &str {
        &self.uid
    }
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.uid);
        hasher.update(self.get_ics());
        format!("{:x}", hasher.finalize())
    }

    pub fn get_ics(&self) -> String {
        self.cal.generate()
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Calendar {
    pub id: String,
    pub name: Option<String>,
    pub owner: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: Option<String>,
}

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(&self, id: &str) -> Result<Calendar>;
    async fn get_calendars(&self, owner: &str) -> Result<Vec<Calendar>>;
    async fn insert_calendar(&mut self, cid: String, calendar: Calendar) -> Result<()>;

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>>;
    async fn get_event(&self, cid: &str, uid: &str) -> Result<Event>;
    async fn upsert_event(&mut self, cid: String, uid: String, ics: String) -> Result<()>;
    async fn delete_event(&mut self, cid: &str, uid: &str) -> Result<()>;
}
