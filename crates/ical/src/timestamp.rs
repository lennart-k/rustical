use super::timezone::ICalTimezone;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use chrono_tz::Tz;
use derive_more::derive::Deref;
use ical::property::Property;
use lazy_static::lazy_static;
use rustical_xml::{ValueDeserialize, ValueSerialize};
use std::{borrow::Cow, collections::HashMap, ops::Add};

lazy_static! {
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

#[derive(Debug, Clone, Deref, PartialEq, Eq, Hash)]
pub struct UtcDateTime(pub DateTime<Utc>);

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalDateTime {
    // Form 1, example: 19980118T230000 -> Local
    // Form 2, example: 19980119T070000Z -> UTC
    // Form 3, example: TZID=America/New_York:19980119T020000 -> Olson
    // https://en.wikipedia.org/wiki/Tz_database
    DateTime(DateTime<ICalTimezone>),
    Date(NaiveDate, ICalTimezone),
}

impl From<CalDateTime> for DateTime<rrule::Tz> {
    fn from(value: CalDateTime) -> Self {
        value
            .as_datetime()
            .into_owned()
            .with_timezone(&value.timezone().to_owned().into())
    }
}

impl From<DateTime<rrule::Tz>> for CalDateTime {
    fn from(value: DateTime<rrule::Tz>) -> Self {
        Self::DateTime(value.with_timezone(&value.timezone().into()))
    }
}

impl PartialOrd for CalDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CalDateTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self, &other) {
            (Self::DateTime(a), Self::DateTime(b)) => a.cmp(b),
            (Self::DateTime(a), Self::Date(..)) => a.cmp(&other.as_datetime()),
            (Self::Date(..), Self::DateTime(b)) => self.as_datetime().as_ref().cmp(b),
            (Self::Date(..), Self::Date(..)) => self.as_datetime().cmp(&other.as_datetime()),
        }
    }
}

impl From<DateTime<Local>> for CalDateTime {
    fn from(value: DateTime<Local>) -> Self {
        CalDateTime::DateTime(value.with_timezone(&ICalTimezone::Local))
    }
}

impl From<DateTime<Utc>> for CalDateTime {
    fn from(value: DateTime<Utc>) -> Self {
        CalDateTime::DateTime(value.with_timezone(&ICalTimezone::Olson(chrono_tz::UTC)))
    }
}

impl Add<Duration> for CalDateTime {
    type Output = Self;

    fn add(self, duration: Duration) -> Self::Output {
        match self {
            Self::DateTime(datetime) => Self::DateTime(datetime + duration),
            Self::Date(date, tz) => Self::DateTime(
                date.and_time(NaiveTime::default())
                    .and_local_timezone(tz)
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
        timezones: &HashMap<String, Option<chrono_tz::Tz>>,
    ) -> Result<Self, CalDateTimeError> {
        let prop_value = prop
            .value
            .as_ref()
            .ok_or(CalDateTimeError::InvalidDatetimeFormat(
                "empty property".to_owned(),
            ))?;

        let timezone = if let Some(tzid) = prop.get_param("TZID") {
            if let Some(timezone) = timezones.get(tzid) {
                timezone.to_owned()
            } else {
                // TZID refers to timezone that does not exist
                return Err(CalDateTimeError::InvalidTZID(tzid.to_string()));
            }
        } else {
            // No explicit timezone specified.
            // This is valid and will be localtime or UTC depending on the value
            // We will stick to this default as documented in https://github.com/lennart-k/rustical/issues/102
            None
        };

        Self::parse(prop_value, timezone)
    }

    pub fn format(&self) -> String {
        match self {
            Self::DateTime(datetime) => match datetime.timezone() {
                ICalTimezone::Olson(chrono_tz::UTC) => datetime.format(UTC_DATE_TIME).to_string(),
                _ => datetime.format(LOCAL_DATE_TIME).to_string(),
            },
            Self::Date(date, _) => date.format(LOCAL_DATE).to_string(),
        }
    }

    pub fn format_date(&self) -> String {
        match self {
            Self::DateTime(datetime) => datetime.format(LOCAL_DATE).to_string(),
            Self::Date(date, _) => date.format(LOCAL_DATE).to_string(),
        }
    }

    pub fn date(&self) -> NaiveDate {
        match self {
            Self::DateTime(datetime) => datetime.date_naive(),
            Self::Date(date, _) => date.to_owned(),
        }
    }

    pub fn is_date(&self) -> bool {
        matches!(&self, Self::Date(_, _))
    }

    pub fn as_datetime(&self) -> Cow<'_, DateTime<ICalTimezone>> {
        match self {
            Self::DateTime(datetime) => Cow::Borrowed(datetime),
            Self::Date(date, tz) => Cow::Owned(
                date.and_time(NaiveTime::default())
                    .and_local_timezone(tz.to_owned())
                    .earliest()
                    .expect("Midnight always exists"),
            ),
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
                    .and_local_timezone(ICalTimezone::Local)
                    .earliest()
                    .ok_or(CalDateTimeError::LocalTimeGap)?,
            ));
        }

        if let Ok(datetime) = NaiveDateTime::parse_from_str(value, UTC_DATE_TIME) {
            return Ok(datetime.and_utc().into());
        }
        let timezone = timezone
            .map(ICalTimezone::Olson)
            .unwrap_or(ICalTimezone::Local);
        if let Ok(date) = NaiveDate::parse_from_str(value, LOCAL_DATE) {
            return Ok(CalDateTime::Date(date, timezone));
        }

        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
            return Ok(CalDateTime::Date(date, timezone));
        }
        if let Ok(date) = NaiveDate::parse_from_str(value, "%Y%m%d") {
            return Ok(CalDateTime::Date(date, timezone));
        }

        Err(CalDateTimeError::InvalidDatetimeFormat(value.to_string()))
    }

    // Also returns whether the date contains a year
    pub fn parse_vcard(value: &str) -> Result<(Self, bool), CalDateTimeError> {
        if let Ok(datetime) = Self::parse(value, None) {
            return Ok((datetime, true));
        }

        if let Some(captures) = RE_VCARD_DATE_MM_DD.captures(value) {
            // Because 1972 is a leap year
            let year = 1972;
            // Cannot fail because of the regex
            let month = captures.name("m").unwrap().as_str().parse().ok().unwrap();
            let day = captures.name("d").unwrap().as_str().parse().ok().unwrap();

            return Ok((
                CalDateTime::Date(
                    NaiveDate::from_ymd_opt(year, month, day)
                        .ok_or(CalDateTimeError::ParseError(value.to_string()))?,
                    ICalTimezone::Local,
                ),
                false,
            ));
        }
        Err(CalDateTimeError::InvalidDatetimeFormat(value.to_string()))
    }

    pub fn utc(&self) -> DateTime<Utc> {
        self.as_datetime().to_utc()
    }

    pub fn timezone(&self) -> ICalTimezone {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.timezone(),
            CalDateTime::Date(_, tz) => tz.to_owned(),
        }
    }
}

impl From<CalDateTime> for DateTime<Utc> {
    fn from(value: CalDateTime) -> Self {
        value.utc()
    }
}

impl Datelike for CalDateTime {
    fn year(&self) -> i32 {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.year(),
            CalDateTime::Date(date, _) => date.year(),
        }
    }
    fn month(&self) -> u32 {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.month(),
            CalDateTime::Date(date, _) => date.month(),
        }
    }

    fn month0(&self) -> u32 {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.month0(),
            CalDateTime::Date(date, _) => date.month0(),
        }
    }
    fn day(&self) -> u32 {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.day(),
            CalDateTime::Date(date, _) => date.day(),
        }
    }
    fn day0(&self) -> u32 {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.day0(),
            CalDateTime::Date(date, _) => date.day0(),
        }
    }
    fn ordinal(&self) -> u32 {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.ordinal(),
            CalDateTime::Date(date, _) => date.ordinal(),
        }
    }
    fn ordinal0(&self) -> u32 {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.ordinal0(),
            CalDateTime::Date(date, _) => date.ordinal0(),
        }
    }
    fn weekday(&self) -> chrono::Weekday {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.weekday(),
            CalDateTime::Date(date, _) => date.weekday(),
        }
    }
    fn iso_week(&self) -> chrono::IsoWeek {
        match &self {
            CalDateTime::DateTime(datetime) => datetime.iso_week(),
            CalDateTime::Date(date, _) => date.iso_week(),
        }
    }
    fn with_year(&self, year: i32) -> Option<Self> {
        match &self {
            CalDateTime::DateTime(datetime) => Some(Self::DateTime(datetime.with_year(year)?)),
            CalDateTime::Date(date, tz) => Some(Self::Date(date.with_year(year)?, tz.to_owned())),
        }
    }
    fn with_month(&self, month: u32) -> Option<Self> {
        match &self {
            CalDateTime::DateTime(datetime) => Some(Self::DateTime(datetime.with_month(month)?)),
            CalDateTime::Date(date, tz) => Some(Self::Date(date.with_month(month)?, tz.to_owned())),
        }
    }
    fn with_month0(&self, month0: u32) -> Option<Self> {
        match &self {
            CalDateTime::DateTime(datetime) => Some(Self::DateTime(datetime.with_month0(month0)?)),
            CalDateTime::Date(date, tz) => {
                Some(Self::Date(date.with_month0(month0)?, tz.to_owned()))
            }
        }
    }
    fn with_day(&self, day: u32) -> Option<Self> {
        match &self {
            CalDateTime::DateTime(datetime) => Some(Self::DateTime(datetime.with_day(day)?)),
            CalDateTime::Date(date, tz) => Some(Self::Date(date.with_day(day)?, tz.to_owned())),
        }
    }
    fn with_day0(&self, day0: u32) -> Option<Self> {
        match &self {
            CalDateTime::DateTime(datetime) => Some(Self::DateTime(datetime.with_day0(day0)?)),
            CalDateTime::Date(date, tz) => Some(Self::Date(date.with_day0(day0)?, tz.to_owned())),
        }
    }
    fn with_ordinal(&self, ordinal: u32) -> Option<Self> {
        match &self {
            CalDateTime::DateTime(datetime) => {
                Some(Self::DateTime(datetime.with_ordinal(ordinal)?))
            }
            CalDateTime::Date(date, tz) => {
                Some(Self::Date(date.with_ordinal(ordinal)?, tz.to_owned()))
            }
        }
    }
    fn with_ordinal0(&self, ordinal0: u32) -> Option<Self> {
        match &self {
            CalDateTime::DateTime(datetime) => {
                Some(Self::DateTime(datetime.with_ordinal0(ordinal0)?))
            }
            CalDateTime::Date(date, tz) => {
                Some(Self::Date(date.with_ordinal0(ordinal0)?, tz.to_owned()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::CalDateTime;
    use chrono::NaiveDate;

    #[test]
    fn test_vcard_date() {
        assert_eq!(
            CalDateTime::parse_vcard("19850412").unwrap(),
            (
                CalDateTime::Date(
                    NaiveDate::from_ymd_opt(1985, 4, 12).unwrap(),
                    crate::ICalTimezone::Local
                ),
                true
            )
        );
        assert_eq!(
            CalDateTime::parse_vcard("1985-04-12").unwrap(),
            (
                CalDateTime::Date(
                    NaiveDate::from_ymd_opt(1985, 4, 12).unwrap(),
                    crate::ICalTimezone::Local
                ),
                true
            )
        );
        assert_eq!(
            CalDateTime::parse_vcard("--0412").unwrap(),
            (
                CalDateTime::Date(
                    NaiveDate::from_ymd_opt(1972, 4, 12).unwrap(),
                    crate::ICalTimezone::Local
                ),
                false
            )
        );
    }
}
