use super::{CalDateTime, EventObject, JournalObject, TodoObject};
use crate::Error;
use anyhow::Result;
use ical::parser::{ical::component::IcalTimeZone, Component};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, io::BufReader};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
// specified in https://datatracker.ietf.org/doc/html/rfc5545#section-3.6
pub enum CalendarObjectType {
    Event = 0,
    Todo = 1,
    Journal = 2,
}

impl rustical_xml::ValueSerialize for CalendarObjectType {
    fn serialize(&self) -> String {
        match self {
            CalendarObjectType::Event => "VEVENT",
            CalendarObjectType::Todo => "VTODO",
            CalendarObjectType::Journal => "VJOURNAL",
        }
        .to_owned()
    }
}

impl rustical_xml::ValueDeserialize for CalendarObjectType {
    fn deserialize(val: &str) -> std::result::Result<Self, rustical_xml::XmlError> {
        match <String as rustical_xml::ValueDeserialize>::deserialize(val)?.as_str() {
            "VEVENT" => Ok(Self::Event),
            "VTODO" => Ok(Self::Todo),
            "VJOURNAL" => Ok(Self::Journal),
            _ => Err(rustical_xml::XmlError::Other(format!(
                "Invalid value '{}', must be VEVENT, VTODO, or VJOURNAL",
                val
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CalendarObjectComponent {
    Event(EventObject),
    Todo(TodoObject),
    Journal(JournalObject),
}

#[derive(Debug, Clone)]
pub struct CalendarObject {
    id: String,
    ics: String,
    data: CalendarObjectComponent,
}

impl CalendarObject {
    pub fn from_ics(object_id: String, ics: String) -> Result<Self, Error> {
        let mut parser = ical::IcalParser::new(BufReader::new(ics.as_bytes()));
        let cal = parser.next().ok_or(Error::NotFound)??;
        if parser.next().is_some() {
            return Err(Error::InvalidData(
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
            return Err(Error::InvalidData(
                "iCalendar object is only allowed to have exactly one component".to_owned(),
            ));
        }

        let timezones: HashMap<String, IcalTimeZone> = cal
            .timezones
            .clone()
            .into_iter()
            .filter_map(|timezone| {
                let timezone_prop = timezone.get_property("TZID")?.to_owned();
                let tzid = timezone_prop.value?;
                Some((tzid, timezone))
            })
            .collect();

        if let Some(event) = cal.events.first() {
            return Ok(CalendarObject {
                id: object_id,
                ics,
                data: CalendarObjectComponent::Event(EventObject {
                    event: event.clone(),
                    timezones,
                }),
            });
        }
        if let Some(todo) = cal.todos.first() {
            return Ok(CalendarObject {
                id: object_id,
                ics,
                data: CalendarObjectComponent::Todo(TodoObject { todo: todo.clone() }),
            });
        }
        if let Some(journal) = cal.journals.first() {
            return Ok(CalendarObject {
                id: object_id,
                ics,
                data: CalendarObjectComponent::Journal(JournalObject {
                    journal: journal.clone(),
                }),
            });
        }

        Err(Error::InvalidData(
            "iCalendar component type not supported :(".to_owned(),
        ))
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.id);
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
            CalendarObjectComponent::Journal(_) => "VJOURNAL",
        }
    }

    pub fn get_object_type(&self) -> CalendarObjectType {
        match self.data {
            CalendarObjectComponent::Todo(_) => CalendarObjectType::Todo,
            CalendarObjectComponent::Event(_) => CalendarObjectType::Event,
            CalendarObjectComponent::Journal(_) => CalendarObjectType::Journal,
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
