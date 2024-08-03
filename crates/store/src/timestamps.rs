use std::{ops::Add, str::FromStr};

use crate::Error;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_DURATION: regex::Regex = regex::Regex::new(r"^(?<sign>[+-])?P((?P<W>\d+)W)?((?P<D>\d+)D)?(T((?P<H>\d+)H)?((?P<M>\d+)M)?((?P<S>\d+)S)?)?$").unwrap();
}

const LOCAL_DATE_TIME: &str = "%Y%m%dT%H%M%S";
const UTC_DATE_TIME: &str = "%Y%m%dT%H%M%SZ";
const LOCAL_DATE: &str = "%Y%m%d";

pub enum CalDateTime {
    // Form 1, example: 19980118T230000
    Local(NaiveDateTime),
    // Form 2, example: 19980119T070000Z
    Utc(DateTime<Utc>),
    // Form 3, example: TZID=America/New_York:19980119T020000
    // TODO: implement timezone parsing
    ExplicitTZ((String, NaiveDateTime)),
    Date(NaiveDate),
}

impl Add<Duration> for CalDateTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        match self {
            Self::Local(datetime) => Self::Local(datetime + duration),
            Self::Utc(datetime) => Self::Utc(datetime + duration),
            Self::ExplicitTZ((tz, datetime)) => Self::ExplicitTZ((tz, datetime + duration)),
            Self::Date(date) => Self::Local(date.and_time(NaiveTime::default()) + duration),
        }
    }
}

impl FromStr for CalDateTime {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, LOCAL_DATE_TIME) {
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
}

#[test]
fn test_parse_cal_datetime() {
    CalDateTime::from_str("19980118T230000").unwrap();
    CalDateTime::from_str("19980118T230000Z").unwrap();
    CalDateTime::from_str("19980118").unwrap();
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
