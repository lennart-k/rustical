use crate::IcalProperty;

use super::timezone::CalTimezone;
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use chrono_tz::Tz;
use derive_more::derive::Deref;
use ical::{
    parser::{Component, ical::component::IcalTimeZone},
    property::Property,
};
use lazy_static::lazy_static;
use rustical_xml::{ValueDeserialize, ValueSerialize};
use std::{collections::HashMap, ops::Add};

lazy_static! {
    static ref RE_DURATION: regex::Regex = regex::Regex::new(r"^(?<sign>[+-])?P((?P<W>\d+)W)?((?P<D>\d+)D)?(T((?P<H>\d+)H)?((?P<M>\d+)M)?((?P<S>\d+)S)?)?$").unwrap();

    static ref RE_VCARD_DATE_MM_DD: regex::Regex =
        regex::Regex::new(r"^--(?<m>\d{2})(?<d>\d{2})$").unwrap();
}

const LOCAL_DATE_TIME: &str = "%Y%m%dT%H%M%S";
const UTC_DATE_TIME: &str = "%Y%m%dT%H%M%SZ";
pub const LOCAL_DATE: &str = "%Y%m%d";

#[derive(Debug, thiserror::Error)]
pub enum CalDateTimeError {
    #[error(
        "Timezone has X-LIC-LOCATION property to specify a timezone from the Olson database, however its value {0} is invalid"
    )]
    InvalidOlson(String),
    #[error("TZID {0} does not refer to a valid timezone")]
    InvalidTZID(String),
    #[error("Timestamp doesn't exist because of gap in local time")]
    LocalTimeGap,
    #[error("Datetime string {0} has an invalid format")]
    InvalidDatetimeFormat(String),
    #[error("Could not parse datetime {0}")]
    ParseError(String),
    #[error("Duration string {0} has an invalid format")]
    InvalidDurationFormat(String),
}

#[derive(Debug, Clone, Deref, PartialEq)]
pub struct UtcDateTime(DateTime<Utc>);

impl ValueDeserialize for UtcDateTime {
    fn deserialize(val: &str) -> Result<Self, rustical_xml::XmlError> {
        let input = <String as ValueDeserialize>::deserialize(val)?;
        Ok(Self(
            NaiveDateTime::parse_from_str(&input, UTC_DATE_TIME)
                .map_err(|_| {
                    rustical_xml::XmlError::InvalidValue(rustical_xml::ParseValueError::Other(
                        "Could not parse as UTC timestamp".to_owned(),
                    ))
                })?
                .and_utc(),
        ))
    }
}

impl ValueSerialize for UtcDateTime {
    fn serialize(&self) -> String {
        format!("{}", self.0.format(UTC_DATE_TIME))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalDateTime {
    // Form 1, example: 19980118T230000 -> Local
    // Form 2, example: 19980119T070000Z -> UTC
    // Form 3, example: TZID=America/New_York:19980119T020000 -> Olson
    // https://en.wikipedia.org/wiki/Tz_database
    DateTime(DateTime<CalTimezone>),
    Date(NaiveDate),
}

impl From<DateTime<Local>> for CalDateTime {
    fn from(value: DateTime<Local>) -> Self {
        CalDateTime::DateTime(value.with_timezone(&CalTimezone::Local))
    }
}

impl From<DateTime<Utc>> for CalDateTime {
    fn from(value: DateTime<Utc>) -> Self {
        CalDateTime::DateTime(value.with_timezone(&CalTimezone::Utc))
    }
}

impl Add<Duration> for CalDateTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        match self {
            Self::DateTime(datetime) => Self::DateTime(datetime + duration),
            Self::Date(date) => Self::DateTime(
                date.and_time(NaiveTime::default())
                    .and_local_timezone(CalTimezone::Local)
                    .earliest()
                    .expect("Local timezone has constant offset")
                    + duration,
            ),
        }
    }
}

impl CalDateTime {
    pub fn parse_prop(
        prop: &Property,
        timezones: &HashMap<String, IcalTimeZone>,
    ) -> Result<Option<Self>, CalDateTimeError> {
        let prop_value = if let Some(value) = prop.value.as_ref() {
            value
        } else {
            return Ok(None);
        };

        // Use the TZID parameter from the property
        let timezone = if let Some(tzid) = prop.get_tzid() {
            if let Some(timezone) = timezones.get(tzid) {
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
                        return Err(CalDateTimeError::InvalidOlson(olson_name));
                    }
                } else {
                    // If the TZID matches a name from the Olson database (e.g. Europe/Berlin) we
                    // guess that we can just use it
                    tzid.parse::<Tz>().ok()
                    // TODO: If None: Too bad, we need to manually parse it
                    // For now it's just treated as localtime
                }
            } else {
                // TZID refers to timezone that does not exist
                return Err(CalDateTimeError::InvalidTZID(tzid.to_string()));
            }
        } else {
            // No explicit timezone specified.
            // This is valid and will be localtime or UTC depending on the value
            None
        };

        Self::parse(prop_value, timezone).map(Some)
    }

    pub fn format(&self) -> String {
        match self {
            Self::DateTime(datetime) => match datetime.timezone() {
                CalTimezone::Utc => datetime.format(UTC_DATE_TIME).to_string(),
                _ => datetime.format(LOCAL_DATE_TIME).to_string(),
            },
            Self::Date(date) => date.format(LOCAL_DATE).to_string(),
        }
    }

    pub fn date(&self) -> NaiveDate {
        match self {
            Self::DateTime(datetime) => datetime.date_naive(),
            Self::Date(date) => date.to_owned(),
        }
    }

    pub fn parse(value: &str, timezone: Option<Tz>) -> Result<Self, CalDateTimeError> {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, LOCAL_DATE_TIME) {
            if let Some(timezone) = timezone {
                return Ok(CalDateTime::DateTime(
                    datetime
                        .and_local_timezone(timezone.into())
                        .earliest()
                        .ok_or(CalDateTimeError::LocalTimeGap)?,
                ));
            }
            return Ok(CalDateTime::DateTime(
                datetime
                    .and_local_timezone(CalTimezone::Local)
                    .earliest()
                    .ok_or(CalDateTimeError::LocalTimeGap)?,
            ));
        }

        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, UTC_DATE_TIME) {
            return Ok(datetime.and_utc().into());
        }
        if let Ok(date) = NaiveDate::parse_from_str(value, LOCAL_DATE) {
            return Ok(CalDateTime::Date(date));
        }

        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            return Ok(CalDateTime::Date(date));
        }
        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y%m%d") {
            return Ok(CalDateTime::Date(date));
        }
        if let Some(captures) = RE_VCARD_DATE_MM_DD.captures(value) {
            // Because 1972 is a leap year
            let year = 1972;
            // Cannot fail because of the regex
            let month = captures.name("m").unwrap().as_str().parse().ok().unwrap();
            let day = captures.name("d").unwrap().as_str().parse().ok().unwrap();

            return Ok(CalDateTime::Date(
                NaiveDate::from_ymd_opt(year, month, day)
                    .ok_or(CalDateTimeError::ParseError(value.to_string()))?,
            ));
        }

        Err(CalDateTimeError::InvalidDatetimeFormat(value.to_string()))
    }

    pub fn utc(&self) -> DateTime<Utc> {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.to_utc(),
            CalDateTime::Date(date) => date.and_time(NaiveTime::default()).and_utc(),
        }
    }
}

impl From<CalDateTime> for DateTime<Utc> {
    fn from(value: CalDateTime) -> Self {
        value.utc()
    }
}

pub fn parse_duration(string: &str) -> Result<Duration, CalDateTimeError> {
    let captures = RE_DURATION
        .captures(string)
        .ok_or(CalDateTimeError::InvalidDurationFormat(string.to_string()))?;

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

#[cfg(test)]
mod tests {
    use crate::{CalDateTime, parse_duration};
    use chrono::{Duration, NaiveDate};

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("P12W").unwrap(), Duration::weeks(12));
        assert_eq!(parse_duration("P12D").unwrap(), Duration::days(12));
        assert_eq!(parse_duration("PT12H").unwrap(), Duration::hours(12));
        assert_eq!(parse_duration("PT12M").unwrap(), Duration::minutes(12));
        assert_eq!(parse_duration("PT12S").unwrap(), Duration::seconds(12));
    }

    #[test]
    fn test_vcard_date() {
        assert_eq!(
            CalDateTime::parse("19850412", None).unwrap(),
            CalDateTime::Date(NaiveDate::from_ymd_opt(1985, 4, 12).unwrap())
        );
        assert_eq!(
            CalDateTime::parse("1985-04-12", None).unwrap(),
            CalDateTime::Date(NaiveDate::from_ymd_opt(1985, 4, 12).unwrap())
        );
        assert_eq!(
            CalDateTime::parse("--0412", None).unwrap(),
            CalDateTime::Date(NaiveDate::from_ymd_opt(1972, 4, 12).unwrap())
        );
    }
}
