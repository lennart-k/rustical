use super::{EventObject, JournalObject, TodoObject};
use crate::CalDateTime;
use crate::Error;
use chrono::DateTime;
use chrono::Utc;
use derive_more::Display;
use ical::generator::{Emitter, IcalCalendar};
use ical::parser::ical::component::IcalTimeZone;
use ical::property::Property;
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, io::BufReader};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display)]
// specified in https://datatracker.ietf.org/doc/html/rfc5545#section-3.6
pub enum CalendarObjectType {
    #[serde(rename = "VEVENT")]
    Event = 0,
    #[serde(rename = "VTODO")]
    Todo = 1,
    #[serde(rename = "VJOURNAL")]
    Journal = 2,
}

impl CalendarObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CalendarObjectType::Event => "VEVENT",
            CalendarObjectType::Todo => "VTODO",
            CalendarObjectType::Journal => "VJOURNAL",
        }
    }
}

impl rustical_xml::ValueSerialize for CalendarObjectType {
    fn serialize(&self) -> String {
        self.as_str().to_owned()
    }
}

impl rustical_xml::ValueDeserialize for CalendarObjectType {
    fn deserialize(val: &str) -> std::result::Result<Self, rustical_xml::XmlError> {
        match <String as rustical_xml::ValueDeserialize>::deserialize(val)?.as_str() {
            "VEVENT" => Ok(Self::Event),
            "VTODO" => Ok(Self::Todo),
            "VJOURNAL" => Ok(Self::Journal),
            _ => Err(rustical_xml::XmlError::InvalidValue(
                rustical_xml::ParseValueError::Other(format!(
                    "Invalid value '{val}', must be VEVENT, VTODO, or VJOURNAL"
                )),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CalendarObjectComponent {
    Event(EventObject),
    Todo(TodoObject),
    Journal(JournalObject),
}

impl Default for CalendarObjectComponent {
    fn default() -> Self {
        Self::Event(EventObject::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct CalendarObject {
    data: CalendarObjectComponent,
    properties: Vec<Property>,
    ics: String,
    vtimezones: HashMap<String, IcalTimeZone>,
}

impl CalendarObject {
    pub fn from_ics(ics: String) -> Result<Self, Error> {
        let mut parser = ical::IcalParser::new(BufReader::new(ics.as_bytes()));
        let cal = parser.next().ok_or(Error::MissingCalendar)??;
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

        let timezones: HashMap<String, Option<chrono_tz::Tz>> = cal
            .timezones
            .clone()
            .into_iter()
            .map(|timezone| (timezone.get_tzid().to_owned(), (&timezone).try_into().ok()))
            .collect();

        let vtimezones = cal
            .timezones
            .clone()
            .into_iter()
            .map(|timezone| (timezone.get_tzid().to_owned(), timezone))
            .collect();

        let data = if let Some(event) = cal.events.into_iter().next() {
            CalendarObjectComponent::Event(EventObject { event, timezones })
        } else if let Some(todo) = cal.todos.into_iter().next() {
            CalendarObjectComponent::Todo(todo.into())
        } else if let Some(journal) = cal.journals.into_iter().next() {
            CalendarObjectComponent::Journal(journal.into())
        } else {
            return Err(Error::InvalidData(
                "iCalendar component type not supported :(".to_owned(),
            ));
        };

        Ok(Self {
            data,
            properties: cal.properties,
            ics,
            vtimezones,
        })
    }

    pub fn get_vtimezones(&self) -> &HashMap<String, IcalTimeZone> {
        &self.vtimezones
    }

    pub fn get_data(&self) -> &CalendarObjectComponent {
        &self.data
    }

    pub fn get_id(&self) -> &str {
        match &self.data {
            CalendarObjectComponent::Todo(todo) => todo.0.get_uid(),
            CalendarObjectComponent::Event(event) => event.event.get_uid(),
            CalendarObjectComponent::Journal(journal) => journal.0.get_uid(),
        }
    }

    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.get_id());
        hasher.update(self.get_ics());
        format!("\"{:x}\"", hasher.finalize())
    }

    pub fn get_ics(&self) -> &str {
        &self.ics
    }

    pub fn get_component_name(&self) -> &str {
        self.get_object_type().as_str()
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
            CalendarObjectComponent::Event(event) => event.get_dtstart(),
            _ => Ok(None),
        }
    }

    pub fn get_last_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        match &self.data {
            CalendarObjectComponent::Event(event) => event.get_last_occurence(),
            _ => Ok(None),
        }
    }

    pub fn event(&self) -> Option<&EventObject> {
        match &self.data {
            CalendarObjectComponent::Event(event) => Some(event),
            _ => None,
        }
    }

    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<String, Error> {
        // Only events can be expanded
        match &self.data {
            CalendarObjectComponent::Event(event) => {
                let cal = IcalCalendar {
                    properties: self.properties.clone(),
                    events: event.expand_recurrence(start, end)?,
                    ..Default::default()
                };
                Ok(cal.generate())
            }
            _ => Ok(self.get_ics().to_string()),
        }
    }
}
