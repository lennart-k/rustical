use std::fmt::Display;

use rustical_xml::ValueSerialize;
use serde::{Deserialize, Serialize};

/// https://datatracker.ietf.org/doc/html/rfc5545#section-3.2.3
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum PrincipalType {
    #[default]
    Individual,
    Group,
    Resource,
    Room,
    Unknown,
    // X-Name, IANA-token
}

impl TryFrom<&str> for PrincipalType {
    type Error = crate::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "INDIVIDUAL" => Self::Individual,
            "GROUP" => Self::Group,
            "RESOURCE" => Self::Resource,
            "ROOM" => Self::Room,
            "UNKNOWN" => Self::Unknown,
            _ => {
                return Err(crate::Error::InvalidPrincipalType(
                    "Invalid principal type".to_owned(),
                ));
            }
        })
    }
}

impl PrincipalType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PrincipalType::Individual => "INDIVIDUAL",
            PrincipalType::Group => "GROUP",
            PrincipalType::Resource => "RESOURCE",
            PrincipalType::Room => "ROOM",
            PrincipalType::Unknown => "UNKNOWN",
        }
    }
}

impl Display for PrincipalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl ValueSerialize for PrincipalType {
    fn serialize(&self) -> String {
        self.to_string()
    }
}
