use super::{event::EventObject, todo::TodoObject};
use crate::{timestamp::CalDateTime, Error};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::BufReader;

#[derive(Debug, Clone)]
// specified in https://datatracker.ietf.org/doc/html/rfc5545#section-3.6
pub enum CalendarObjectType {
    Event,
    Journal,
    Todo,
}

#[derive(Debug, Clone)]
pub enum CalendarObjectComponent {
    Event(EventObject),
    Todo(TodoObject),
}

#[derive(Debug, Clone)]
pub struct CalendarObject {
    uid: String,
    ics: String,
    data: CalendarObjectComponent,
}

// Custom implementation for CalendarObject (de)serialization
impl<'de> Deserialize<'de> for CalendarObject {
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

impl Serialize for CalendarObject {
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

impl CalendarObject {
    pub fn from_ics(uid: String, ics: String) -> Result<Self, Error> {
        let mut parser = ical::IcalParser::new(BufReader::new(ics.as_bytes()));
        let cal = parser.next().ok_or(Error::NotFound)??;
        if parser.next().is_some() {
            return Err(Error::InvalidIcs(
                "multiple calendars, only one allowed".to_owned(),
            ));
        }
        if cal.events.len()
            + cal.alarms.len()
            + cal.todos.len()
            + cal.journals.len()
            + cal.free_busys.len()
            != 1
        {
            // https://datatracker.ietf.org/doc/html/rfc4791#section-4.1
            return Err(Error::InvalidIcs(
                "iCalendar object is only allowed to have exactly one component".to_owned(),
            ));
        }

        if let Some(event) = cal.events.first() {
            return Ok(CalendarObject {
                uid,
                ics,
                data: CalendarObjectComponent::Event(EventObject {
                    event: event.clone(),
                }),
            });
        }
        if let Some(todo) = cal.todos.first() {
            return Ok(CalendarObject {
                uid,
                ics,
                data: CalendarObjectComponent::Todo(TodoObject { todo: todo.clone() }),
            });
        }

        Err(Error::InvalidIcs(
            "iCalendar component type not supported :(".to_owned(),
        ))
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

    pub fn get_ics(&self) -> &str {
        &self.ics
    }

    pub fn get_component_name(&self) -> &str {
        match self.data {
            CalendarObjectComponent::Todo(_) => "VTODO",
            CalendarObjectComponent::Event(_) => "VEVENT",
        }
    }

    pub fn get_first_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        match &self.data {
            CalendarObjectComponent::Event(event) => event.get_first_occurence(),
            _ => Ok(None),
        }
    }

    pub fn get_last_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        match &self.data {
            CalendarObjectComponent::Event(event) => event.get_last_occurence(),
            _ => Ok(None),
        }
    }
}
