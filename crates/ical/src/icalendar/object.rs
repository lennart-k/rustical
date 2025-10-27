use super::EventObject;
use crate::CalDateTime;
use crate::Error;
use chrono::DateTime;
use chrono::Utc;
use derive_more::Display;
use ical::generator::{Emitter, IcalCalendar};
use ical::parser::ical::component::IcalJournal;
use ical::parser::ical::component::IcalTimeZone;
use ical::parser::ical::component::IcalTodo;
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
    #[must_use] pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Event => "VEVENT",
            Self::Todo => "VTODO",
            Self::Journal => "VJOURNAL",
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
    Event(EventObject, Vec<EventObject>),
    Todo(IcalTodo, Vec<IcalTodo>),
    Journal(IcalJournal, Vec<IcalJournal>),
}

impl From<&CalendarObjectComponent> for CalendarObjectType {
    fn from(value: &CalendarObjectComponent) -> Self {
        match value {
            CalendarObjectComponent::Event(..) => Self::Event,
            CalendarObjectComponent::Todo(..) => Self::Todo,
            CalendarObjectComponent::Journal(..) => Self::Journal,
        }
    }
}

impl CalendarObjectComponent {
    fn from_events(mut events: Vec<EventObject>) -> Result<Self, Error> {
        let main_event = events
            .extract_if(.., |event| event.event.get_recurrence_id().is_none())
            .next()
            .expect("there must be one main event");
        let overrides = events;
        for event in &overrides {
            if event.get_uid() != main_event.get_uid() {
                return Err(Error::InvalidData(
                    "Calendar object contains multiple UIDs".to_owned(),
                ));
            }
            if event.event.get_recurrence_id().is_none() {
                return Err(Error::InvalidData(
                    "Calendar object can only contain one main component".to_owned(),
                ));
            }
        }
        Ok(Self::Event(main_event, overrides))
    }
    fn from_todos(mut todos: Vec<IcalTodo>) -> Result<Self, Error> {
        let main_todo = todos
            .extract_if(.., |todo| todo.get_recurrence_id().is_none())
            .next()
            .expect("there must be one main event");
        let overrides = todos;
        for todo in &overrides {
            if todo.get_uid() != main_todo.get_uid() {
                return Err(Error::InvalidData(
                    "Calendar object contains multiple UIDs".to_owned(),
                ));
            }
            if todo.get_recurrence_id().is_none() {
                return Err(Error::InvalidData(
                    "Calendar object can only contain one main component".to_owned(),
                ));
            }
        }
        Ok(Self::Todo(main_todo, overrides))
    }
    fn from_journals(mut journals: Vec<IcalJournal>) -> Result<Self, Error> {
        let main_journal = journals
            .extract_if(.., |journal| journal.get_recurrence_id().is_none())
            .next()
            .expect("there must be one main event");
        let overrides = journals;
        for journal in &overrides {
            if journal.get_uid() != main_journal.get_uid() {
                return Err(Error::InvalidData(
                    "Calendar object contains multiple UIDs".to_owned(),
                ));
            }
            if journal.get_recurrence_id().is_none() {
                return Err(Error::InvalidData(
                    "Calendar object can only contain one main component".to_owned(),
                ));
            }
        }
        Ok(Self::Journal(main_journal, overrides))
    }
}

#[derive(Debug, Clone)]
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

        if u8::from(!cal.events.is_empty())
            + u8::from(!cal.todos.is_empty())
            + u8::from(!cal.journals.is_empty())
            + u8::from(!cal.free_busys.is_empty())
            != 1
        {
            // https://datatracker.ietf.org/doc/html/rfc4791#section-4.1
            return Err(Error::InvalidData(
                "iCalendar object must have exactly one component type".to_owned(),
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

        let data = if !cal.events.is_empty() {
            CalendarObjectComponent::from_events(
                cal.events
                    .into_iter()
                    .map(|event| EventObject {
                        event,
                        timezones: timezones.clone(),
                    })
                    .collect(),
            )?
        } else if !cal.todos.is_empty() {
            CalendarObjectComponent::from_todos(cal.todos)?
        } else if !cal.journals.is_empty() {
            CalendarObjectComponent::from_journals(cal.journals)?
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

    #[must_use] pub const fn get_vtimezones(&self) -> &HashMap<String, IcalTimeZone> {
        &self.vtimezones
    }

    #[must_use] pub const fn get_data(&self) -> &CalendarObjectComponent {
        &self.data
    }

    #[must_use] pub fn get_id(&self) -> &str {
        match &self.data {
            // We've made sure before that the first component exists and all components share the
            // same UID
            CalendarObjectComponent::Todo(todo, _) => todo.get_uid(),
            CalendarObjectComponent::Event(event, _) => event.event.get_uid(),
            CalendarObjectComponent::Journal(journal, _) => journal.get_uid(),
        }
    }

    #[must_use] pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.get_id());
        hasher.update(self.get_ics());
        format!("\"{:x}\"", hasher.finalize())
    }

    #[must_use] pub fn get_ics(&self) -> &str {
        &self.ics
    }

    #[must_use] pub fn get_component_name(&self) -> &str {
        self.get_object_type().as_str()
    }

    #[must_use] pub fn get_object_type(&self) -> CalendarObjectType {
        (&self.data).into()
    }

    pub fn get_first_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        match &self.data {
            CalendarObjectComponent::Event(main_event, overrides) => Ok(overrides
                .iter()
                .chain([main_event].into_iter())
                .map(super::event::EventObject::get_dtstart)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .min()),
            _ => Ok(None),
        }
    }

    pub fn get_last_occurence(&self) -> Result<Option<CalDateTime>, Error> {
        match &self.data {
            CalendarObjectComponent::Event(main_event, overrides) => Ok(overrides
                .iter()
                .chain([main_event].into_iter())
                .map(super::event::EventObject::get_last_occurence)
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten()
                .max()),
            _ => Ok(None),
        }
    }

    pub fn expand_recurrence(
        &self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Result<String, Error> {
        // Only events can be expanded
        match &self.data {
            CalendarObjectComponent::Event(main_event, overrides) => {
                let cal = IcalCalendar {
                    properties: self.properties.clone(),
                    events: main_event.expand_recurrence(start, end, overrides)?,
                    ..Default::default()
                };
                Ok(cal.generate())
            }
            _ => Ok(self.get_ics().to_string()),
        }
    }
}
