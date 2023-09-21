use std::io::BufReader;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{Duration, NaiveDateTime, Timelike};
use ical::{
    generator::{Emitter, IcalCalendar},
    parser::Component,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

lazy_static! {
    static ref RE_DURATION: regex::Regex = regex::Regex::new(r"^(?<sign>[+-])?P((?P<W>\d+)W)?((?P<D>\d+)D)?(T((?P<H>\d+)H)?((?P<M>\d+)M)?((?P<S>\d+)S)?)?$").unwrap();
}

#[derive(Debug, Clone)]
pub struct Event {
    uid: String,
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
    pub fn from_ics(uid: String, ics: String) -> Result<Self> {
        let mut parser = ical::IcalParser::new(BufReader::new(ics.as_bytes()));
        let cal = parser.next().ok_or(anyhow!("no calendar :("))??;
        if parser.next().is_some() {
            return Err(anyhow!("multiple calendars!"));
        }
        if cal.events.len() != 1 {
            return Err(anyhow!("multiple or no events"));
        }
        let event = Self { uid, cal };
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

    pub fn get_ics(&self) -> String {
        self.cal.generate()
    }

    pub fn get_first_occurence(&self) -> Result<NaiveDateTime> {
        // This is safe since we enforce the event's existance in the constructor
        let event = &self.cal.events.get(0).unwrap();
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
        let event = &self.cal.events.get(0).unwrap();

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

pub fn parse_duration(string: &str) -> Result<Duration> {
    let captures = RE_DURATION
        .captures(string)
        .ok_or(anyhow!("invalid duration format"))?;

    let mut duration = Duration::zero();
    if let Some(weeks) = captures.name("W") {
        duration = duration + Duration::weeks(weeks.as_str().parse()?);
    }
    if let Some(days) = captures.name("D") {
        duration = duration + Duration::days(days.as_str().parse()?);
    }
    if let Some(hours) = captures.name("H") {
        duration = duration + Duration::hours(hours.as_str().parse()?);
    }
    if let Some(minutes) = captures.name("M") {
        duration = duration + Duration::minutes(minutes.as_str().parse()?);
    }
    if let Some(seconds) = captures.name("S") {
        duration = duration + Duration::seconds(seconds.as_str().parse()?);
    }
    if let Some(sign) = captures.name("sign") {
        if sign.as_str() == "-" {
            duration = -duration;
        }
    }

    Ok(duration)
}

#[test]
pub fn test_parse_duration() {
    assert_eq!(parse_duration("P12W").unwrap(), Duration::weeks(12));
    assert_eq!(parse_duration("P12D").unwrap(), Duration::days(12));
    assert_eq!(parse_duration("PT12H").unwrap(), Duration::hours(12));
    assert_eq!(parse_duration("PT12M").unwrap(), Duration::minutes(12));
    assert_eq!(parse_duration("PT12S").unwrap(), Duration::seconds(12));
}

pub fn parse_datetime(string: &str) -> Result<NaiveDateTime> {
    // TODO: respect timezones
    //
    // Format: ^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})(?P<utc>Z)?$
    // if Z?
    //   UTC time
    // else
    //   if TZID given?
    //      time in TZ
    //   else
    //      local time of attendee (can be different actual times for different attendees)
    //      BUT for this implementation will be UTC for now since this case is annoying
    //      (sabre-dav does same)
    let (datetime, _tz_remainder) = NaiveDateTime::parse_and_remainder(string, "%Y%m%dT%H%M%S")?;
    Ok(datetime)
}

#[test]
fn test_parse_datetime() {
    dbg!(parse_datetime("19960329T133000Z").unwrap());
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct Calendar {
    pub id: String,
    pub name: Option<String>,
    pub owner: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub timezone: Option<String>,
}

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    async fn get_calendar(&self, id: &str) -> Result<Calendar>;
    async fn get_calendars(&self, owner: &str) -> Result<Vec<Calendar>>;
    async fn insert_calendar(&mut self, cid: String, calendar: Calendar) -> Result<()>;

    async fn get_events(&self, cid: &str) -> Result<Vec<Event>>;
    async fn get_event(&self, cid: &str, uid: &str) -> Result<Event>;
    async fn upsert_event(&mut self, cid: String, uid: String, ics: String) -> Result<()>;
    async fn delete_event(&mut self, cid: &str, uid: &str) -> Result<()>;
}
