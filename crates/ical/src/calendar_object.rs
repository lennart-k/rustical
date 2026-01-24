use std::sync::OnceLock;

use crate::Error;
use caldata::{
    IcalObjectParser,
    component::{CalendarInnerData, IcalCalendarObject},
    generator::Emitter,
    parser::ParserOptions,
};
use derive_more::Display;
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
    inner: IcalCalendarObject,
    ics: OnceLock<String>,
}

impl CalendarObject {
    // This function parses iCalendar data but doesn't cache it
    // This is meant for iCalendar data coming from outside that might need to be normalised.
    // For example if timezones are omitted this can be fixed by this function.
    pub fn import(ics: &str, options: Option<ParserOptions>) -> Result<Self, Error> {
        let parser =
            IcalObjectParser::from_slice(ics.as_bytes()).with_options(options.unwrap_or_default());
        let inner = parser.expect_one()?;

        Ok(Self {
            inner,
            ics: OnceLock::new(),
        })
    }

    // This function parses iCalendar data and then caches the parsed iCalendar data.
    // This function is only meant for loading data from a data store where we know the iCalendar
    // is already in the desired form.
    pub fn from_ics(ics: String) -> Result<Self, Error> {
        let parser = IcalObjectParser::from_slice(ics.as_bytes());
        let inner = parser.expect_one()?;

        Ok(Self {
            inner,
            ics: ics.into(),
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
    pub fn get_etag(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.get_uid());
        hasher.update(self.get_ics());
        format!("\"{:x}\"", hasher.finalize())
    }

    #[must_use]
    pub fn get_ics(&self) -> &str {
        self.ics.get_or_init(|| self.inner.generate())
    }

    #[must_use]
    pub fn get_object_type(&self) -> CalendarObjectType {
        (&self.inner).into()
    }
}

impl From<CalendarObject> for IcalCalendarObject {
    fn from(value: CalendarObject) -> Self {
        value.inner
    }
}

impl From<IcalCalendarObject> for CalendarObject {
    fn from(value: IcalCalendarObject) -> Self {
        Self {
            ics: value.generate().into(),
            inner: value,
        }
    }
}
