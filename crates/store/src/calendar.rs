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
    pub ics: String,
}

impl Calendar {}

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(&self, id: &str) -> Result<Calendar>;
    async fn get_calendars(&self) -> Result<Vec<Calendar>>;

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>>;
    async fn get_event(&self, uid: &str) -> Result<Event>;
    async fn upsert_event(&mut self, uid: String, ics: String) -> Result<()>;
    async fn delete_event(&mut self, uid: &str) -> Result<()>;
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
impl CalendarStore for JsonCalendarStore {
    async fn get_calendar(&self, id: &str) -> Result<Calendar> {
        Ok(self.calendars.get(id).ok_or(anyhow!("not found"))?.clone())
    }

    async fn get_calendars(&self) -> Result<Vec<Calendar>> {
        Ok(vec![Calendar {
            id: "test".to_string(),
            name: Some("test".to_string()),
            ics: "asd".to_string(),
        }])
    }

    async fn get_events(&self, _cid: &str) -> Result<Vec<Event>> {
        Ok(self.events.values().cloned().collect())
    }

    async fn get_event(&self, uid: &str) -> Result<Event> {
        Ok(self.events.get(uid).ok_or(anyhow!("not found"))?.clone())
    }

    async fn upsert_event(&mut self, uid: String, ics: String) -> Result<()> {
        self.events.insert(uid.clone(), Event { uid, ics });
        self.save().await.unwrap();
        Ok(())
    }

    async fn delete_event(&mut self, uid: &str) -> Result<()> {
        self.events.remove(uid);
        self.save().await?;
        Ok(())
    }
}
