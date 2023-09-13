use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Event {
    uid: String,
    ics: String,
}

impl Event {
    pub fn from_ics(uid: String, ics: String) -> Self {
        Self { uid, ics }
    }
    pub fn get_uid(&self) -> &str {
        &self.uid
    }
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.uid);
        hasher.update(self.to_ics());
        format!("{:x}", hasher.finalize())
    }

    pub fn to_ics(&self) -> &str {
        &self.ics
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

impl Calendar {}

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
