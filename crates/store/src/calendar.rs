use std::collections::HashMap;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Event {
    uid: String,
    ics: String,
}

impl Event {
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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Calendar {
    pub id: String,
    pub name: Option<String>,
    pub owner: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub ics: String,
}

impl Calendar {}

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(&self, id: &str) -> Result<Calendar>;
    async fn get_calendars(&self, owner: &str) -> Result<Vec<Calendar>>;

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>>;
    async fn get_event(&self, cid: &str, uid: &str) -> Result<Event>;
    async fn upsert_event(&mut self, cid: String, uid: String, ics: String) -> Result<()>;
    async fn delete_event(&mut self, cid: &str, uid: &str) -> Result<()>;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TomlCalendarStore {
    calendars: HashMap<String, Calendar>,
    events: HashMap<String, HashMap<String, Event>>,
    path: String,
}

impl TomlCalendarStore {
    pub fn new(path: String) -> Self {
        TomlCalendarStore {
            calendars: HashMap::new(),
            events: HashMap::new(),
            path,
        }
    }

    pub async fn save(&self) -> Result<()> {
        let mut file = File::create(&self.path).await?;
        let output = toml::to_string_pretty(&self)?;
        file.write_all(output.as_bytes()).await?;
        Ok(())
    }
}

#[async_trait]
impl CalendarStore for TomlCalendarStore {
    async fn get_calendar(&self, id: &str) -> Result<Calendar> {
        Ok(self.calendars.get(id).ok_or(anyhow!("not found"))?.clone())
    }

    async fn get_calendars(&self, user: &str) -> Result<Vec<Calendar>> {
        Ok(self
            .calendars
            .values()
            .filter(|Calendar { owner, .. }| owner == user)
            .cloned()
            .collect())
    }

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>> {
        if let Some(events) = self.events.get(cid) {
            Ok(events.values().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_event(&self, cid: &str, uid: &str) -> Result<Event> {
        let events = self.events.get(cid).ok_or(anyhow!("not found"))?;
        Ok(events.get(uid).ok_or(anyhow!("not found"))?.clone())
    }

    async fn upsert_event(&mut self, cid: String, uid: String, ics: String) -> Result<()> {
        let events = self.events.entry(cid).or_insert(HashMap::new());
        events.insert(
            uid.clone(),
            Event {
                uid,
                ics,
                summary: None,
            },
        );
        self.save().await.unwrap();
        Ok(())
    }

    async fn delete_event(&mut self, cid: &str, uid: &str) -> Result<()> {
        if let Some(events) = self.events.get_mut(cid) {
            events.remove(uid);
            self.save().await?;
        }
        Ok(())
    }
}
