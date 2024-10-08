use crate::Error;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use chrono_tz::Tz;
use ical::{
    parser::{ical::component::IcalTimeZone, Component},
    property::Property,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, ops::Add};

lazy_static! {
    static ref RE_DURATION: regex::Regex = regex::Regex::new(r"^(?<sign>[+-])?P((?P<W>\d+)W)?((?P<D>\d+)D)?(T((?P<H>\d+)H)?((?P<M>\d+)M)?((?P<S>\d+)S)?)?$").unwrap();
}

const LOCAL_DATE_TIME: &str = "%Y%m%dT%H%M%S";
const UTC_DATE_TIME: &str = "%Y%m%dT%H%M%SZ";
const LOCAL_DATE: &str = "%Y%m%d";

#[derive(Debug, Clone)]
pub enum CalDateTime {
    // Form 1, example: 19980118T230000
    Local(NaiveDateTime),
    // Form 2, example: 19980119T070000Z
    Utc(DateTime<Utc>),
    // Form 3, example: TZID=America/New_York:19980119T020000
    // https://en.wikipedia.org/wiki/Tz_database
    OlsonTZ(DateTime<Tz>),
    Date(NaiveDate),
}

pub fn deserialize_utc_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    type Inner = Option<String>;
    Ok(if let Some(input) = Inner::deserialize(deserializer)? {
        Some(
            NaiveDateTime::parse_from_str(&input, UTC_DATE_TIME)
                .map_err(|err| serde::de::Error::custom(err.to_string()))?
                .and_utc(),
        )
    } else {
        None
    })
}

impl Add<Duration> for CalDateTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        match self {
            Self::Local(datetime) => Self::Local(datetime + duration),
            Self::Utc(datetime) => Self::Utc(datetime + duration),
            Self::OlsonTZ(datetime) => Self::OlsonTZ(datetime + duration),
            Self::Date(date) => Self::Local(date.and_time(NaiveTime::default()) + duration),
        }
    }
}

impl CalDateTime {
    pub fn parse_prop(
        prop: &Property,
        timezones: &HashMap<String, IcalTimeZone>,
    ) -> Result<Option<Self>, Error> {
        let prop_value = if let Some(value) = &prop.value {
            value.to_owned()
        } else {
            return Ok(None);
        };

        let timezone = if let Some(tzid) = &prop
            .params
            .clone()
            .unwrap_or_default()
            .iter()
            .filter(|(name, _values)| name == "TZID")
            .map(|(_name, values)| values.first())
            .next()
            .unwrap_or_default()
        {
            if let Some(timezone) = timezones.get(tzid.to_owned()) {
                // X-LIC-LOCATION is often used to refer to a standardised timezone from the Olson
                // database
                if let Some(olson_name) = timezone
                    .get_property("X-LIC-LOCATION")
                    .map(|prop| prop.value.to_owned())
                    .unwrap_or_default()
                {
                    if let Ok(tz) = olson_name.parse::<Tz>() {
                        Some(tz)
                    } else {
                        // TODO: handle invalid timezone name
                        None
                    }
                } else {
                    // No name, we would have to parse it ourselves :(
                    // TODO: implement
                    None
                }
            } else {
                // ERROR: invalid timezone specified
                // For now just assume naive datetime?
                None
            }
        } else {
            None
        };

        Self::parse(&prop_value, timezone).map(Some)
    }

    pub fn parse(value: &str, timezone: Option<Tz>) -> Result<Self, Error> {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, LOCAL_DATE_TIME) {
            if let Some(timezone) = timezone {
                let result = datetime.and_local_timezone(timezone);
                if let Some(datetime) = result.earliest() {
                    return Ok(CalDateTime::OlsonTZ(datetime));
                } else {
                    // This time does not exist because there's a gap in local time
                    return Err(Error::Other(anyhow!(
                        "Timestamp doesn't exist because of gap in local time"
                    )));
                }
            }
            return Ok(CalDateTime::Local(datetime));
        }

        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, UTC_DATE_TIME) {
            return Ok(CalDateTime::Utc(datetime.and_utc()));
        }
        if let Ok(date) = NaiveDate::parse_from_str(value, LOCAL_DATE) {
            return Ok(CalDateTime::Date(date));
        }

        Err(Error::Other(anyhow!("Invalid datetime format")))
    }

    pub fn utc(&self) -> DateTime<Utc> {
        match &self {
            CalDateTime::Local(local_datetime) => local_datetime.and_utc(),
            CalDateTime::Utc(utc_datetime) => utc_datetime.to_owned(),
            CalDateTime::OlsonTZ(datetime) => datetime.to_utc(),
            CalDateTime::Date(date) => date.and_time(NaiveTime::default()).and_utc(),
        }
    }
}

impl From<CalDateTime> for DateTime<Utc> {
    fn from(value: CalDateTime) -> Self {
        value.utc()
    }
}

pub fn parse_duration(string: &str) -> Result<Duration, Error> {
    let captures = RE_DURATION
        .captures(string)
        .ok_or(Error::InvalidIcs("Invalid duration format".to_owned()))?;

    let mut duration = Duration::zero();
    if let Some(weeks) = captures.name("W") {
        duration += Duration::weeks(weeks.as_str().parse().unwrap());
    }
    if let Some(days) = captures.name("D") {
        duration += Duration::days(days.as_str().parse().unwrap());
    }
    if let Some(hours) = captures.name("H") {
        duration += Duration::hours(hours.as_str().parse().unwrap());
    }
    if let Some(minutes) = captures.name("M") {
        duration += Duration::minutes(minutes.as_str().parse().unwrap());
    }
    if let Some(seconds) = captures.name("S") {
        duration += Duration::seconds(seconds.as_str().parse().unwrap());
    }
    if let Some(sign) = captures.name("sign") {
        if sign.as_str() == "-" {
            duration = -duration;
        }
    }

    Ok(duration)
}

#[test]
fn test_parse_duration() {
    assert_eq!(parse_duration("P12W").unwrap(), Duration::weeks(12));
    assert_eq!(parse_duration("P12D").unwrap(), Duration::days(12));
    assert_eq!(parse_duration("PT12H").unwrap(), Duration::hours(12));
    assert_eq!(parse_duration("PT12M").unwrap(), Duration::minutes(12));
    assert_eq!(parse_duration("PT12S").unwrap(), Duration::seconds(12));
}
