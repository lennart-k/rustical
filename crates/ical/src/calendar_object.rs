use crate::Error;
use derive_more::Display;
use ical::component::CalendarInnerData;
use ical::component::IcalCalendarObject;
use ical::parser::ComponentParser;
use serde::Deserialize;
use serde::Serialize;
use sha2::{Digest, Sha256};

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

impl From<&IcalCalendarObject> for CalendarObjectType {
    fn from(value: &IcalCalendarObject) -> Self {
        match value.get_inner() {
            CalendarInnerData::Event(_, _) => Self::Event,
            CalendarInnerData::Todo(_, _) => Self::Todo,
            CalendarInnerData::Journal(_, _) => Self::Journal,
        }
    }
}

impl CalendarObjectType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
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
pub struct CalendarObject {
    id: String,
    inner: IcalCalendarObject,
    ics: String,
}

impl CalendarObject {
    pub fn from_ics(ics: String, id: Option<String>) -> Result<Self, Error> {
        let mut parser: ComponentParser<_, IcalCalendarObject> =
            ComponentParser::new(ics.as_bytes());
        let inner = parser.next().ok_or(Error::MissingCalendar)??;
        if parser.next().is_some() {
            return Err(Error::InvalidData(
                "multiple calendars, only one allowed".to_owned(),
            ));
        }

        Ok(Self {
            id: id.unwrap_or_else(|| inner.get_uid().to_owned()),
            inner,
            ics,
        })
    }

    #[must_use]
    pub const fn get_inner(&self) -> &IcalCalendarObject {
        &self.inner
    }

    #[must_use]
    pub fn get_uid(&self) -> &str {
        self.inner.get_uid()
    }

    #[must_use]
    pub fn get_id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.get_uid());
        hasher.update(self.get_ics());
        format!("\"{:x}\"", hasher.finalize())
    }

    #[must_use]
    pub fn get_ics(&self) -> &str {
        &self.ics
    }

    #[must_use]
    pub fn get_object_type(&self) -> CalendarObjectType {
        (&self.inner).into()
    }
}
