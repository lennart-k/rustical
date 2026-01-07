use chrono::{DateTime, NaiveDateTime, Utc};
use derive_more::derive::Deref;
use rustical_xml::{ValueDeserialize, ValueSerialize};

const LOCAL_DATE_TIME: &str = "%Y%m%dT%H%M%S";
const UTC_DATE_TIME: &str = "%Y%m%dT%H%M%SZ";
pub const LOCAL_DATE: &str = "%Y%m%d";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
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
