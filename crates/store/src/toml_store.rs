use crate::calendar::{Calendar, CalendarStore};
use crate::event::Event;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, HashMap};
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Debug, Deserialize, Serialize)]
pub struct TomlCalendarStore {
    calendars: HashMap<String, Calendar>,
    events: HashMap<String, HashMap<String, Event>>,
    path: Option<String>,
}

impl TomlCalendarStore {
    pub fn new(path: String) -> Self {
        TomlCalendarStore {
            calendars: HashMap::new(),
            events: HashMap::new(),
            path: Some(path),
        }
    }

    pub fn test() -> Self {
        TomlCalendarStore {
            calendars: HashMap::new(),
            events: HashMap::new(),
            path: None,
        }
    }

    pub async fn save(&self) -> Result<()> {
        let output = toml::to_string_pretty(&self)?;
        if let Some(path) = &self.path {
            let mut file = File::create(path).await?;
            file.write_all(output.as_bytes()).await?;
        }
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

    async fn insert_calendar(&mut self, cid: String, calendar: Calendar) -> Result<()> {
        match self.calendars.entry(cid) {
            Entry::Occupied(_) => Err(anyhow!("calendar already exists")),
            Entry::Vacant(v) => {
                v.insert(calendar);
                self.save().await.unwrap();
                Ok(())
            }
        }
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
        let events = self.events.entry(cid).or_default();
        events.insert(uid.clone(), Event::from_ics(uid, ics)?);
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
