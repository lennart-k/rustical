use chrono::{DateTime, NaiveDateTime, Utc};
use derive_more::derive::Deref;
use rustical_xml::{ValueDeserialize, ValueSerialize};

const UTC_DATE_TIME: &str = "%Y%m%dT%H%M%SZ";

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
