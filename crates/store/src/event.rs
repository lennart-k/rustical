use crate::{
    timestamps::{parse_datetime, parse_duration},
    Error,
};
use anyhow::{anyhow, Result};
use chrono::{Duration, NaiveDateTime, Timelike};
use ical::parser::{ical::component::IcalCalendar, Component};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::BufReader;

#[derive(Debug, Clone)]
pub struct Event {
    uid: String,
    ics: String,
    cal: IcalCalendar,
}

// Custom implementation for Event (de)serialization
impl<'de> Deserialize<'de> for Event {
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
impl Serialize for Event {
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

impl Event {
    // https://datatracker.ietf.org/doc/html/rfc4791#section-4.1
    // MUST NOT contain more than one calendar objects (VEVENT, VTODO, VJOURNAL)
    pub fn from_ics(uid: String, ics: String) -> Result<Self, Error> {
        let mut parser = ical::IcalParser::new(BufReader::new(ics.as_bytes()));
        let cal = parser.next().ok_or(Error::NotFound)??;
        if parser.next().is_some() {
            return Err(anyhow!("multiple calendars!").into());
        }
        if cal.events.len() != 1 {
            return Err(anyhow!("multiple or no events").into());
        }
        let event = Self { uid, cal, ics };
        // Run getters now to validate the input and ensure that they'll work later on
        event.get_first_occurence()?;
        event.get_last_occurence()?;
        Ok(event)
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

    pub fn get_first_occurence(&self) -> Result<NaiveDateTime> {
        // This is safe since we enforce the event's existance in the constructor
        let event = self.cal.events.first().unwrap();
        let dtstart = event
            .get_property("DTSTART")
            .ok_or(anyhow!("DTSTART property missing!"))?
            .value
            .to_owned()
            .ok_or(anyhow!("DTSTART property has no value!"))?;
        parse_datetime(&dtstart)
    }

    pub fn get_last_occurence(&self) -> Result<NaiveDateTime> {
        // This is safe since we enforce the event's existance in the constructor
        let event = self.cal.events.first().unwrap();

        if event.get_property("RRULE").is_some() {
            // TODO: understand recurrence rules
            return Err(anyhow!("event is recurring, we cannot handle that yet"));
        }

        if let Some(dtend_prop) = event.get_property("DTEND") {
            let dtend = dtend_prop
                .value
                .to_owned()
                .ok_or(anyhow!("DTEND property has no value!"))?;
            return parse_datetime(&dtend);
        }

        if let Some(dtend_prop) = event.get_property("DURATION") {
            let duration = dtend_prop
                .value
                .to_owned()
                .ok_or(anyhow!("DURATION property has no value!"))?;
            let dtstart = self.get_first_occurence()?;
            return Ok(dtstart + parse_duration(&duration)?);
        }

        let dtstart = self.get_first_occurence()?;
        if dtstart.num_seconds_from_midnight() == 0 {
            // no explicit time given => whole-day event
            return Ok(dtstart + Duration::days(1));
        };

        Err(anyhow!("help, couldn't determine any last occurence"))
    }
}
