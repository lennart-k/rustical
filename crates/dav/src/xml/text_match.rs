use ical::property::Property;
use rustical_xml::{ValueDeserialize, XmlDeserialize};
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum TextCollation {
    #[default]
    AsciiCasemap,
    UnicodeCasemap,
    Octet,
}

impl TextCollation {
    #[must_use]
    pub fn normalise<'a>(&self, value: &'a str) -> Cow<'a, str> {
        match self {
            // https://datatracker.ietf.org/doc/html/rfc4790#section-9.2
            Self::AsciiCasemap => Cow::from(value.to_ascii_uppercase()),
            Self::UnicodeCasemap => Cow::from(value.to_uppercase()),
            Self::Octet => Cow::from(value),
        }
    }
}

impl AsRef<str> for TextCollation {
    fn as_ref(&self) -> &str {
        match self {
            Self::AsciiCasemap => "i;ascii-casemap",
            Self::UnicodeCasemap => "i;unicode-casemap",
            Self::Octet => "i;octet",
        }
    }
}

impl ValueDeserialize for TextCollation {
    fn deserialize(val: &str) -> Result<Self, rustical_xml::XmlError> {
        match val {
            "i;ascii-casemap" => Ok(Self::AsciiCasemap),
            "i;unicode-casemap" => Ok(Self::UnicodeCasemap),
            "i;octet" => Ok(Self::Octet),
            _ => Err(rustical_xml::XmlError::InvalidVariant(format!(
                "Invalid collation: {val}"
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct NegateCondition(pub bool);

impl ValueDeserialize for NegateCondition {
    fn deserialize(val: &str) -> Result<Self, rustical_xml::XmlError> {
        match val {
            "yes" => Ok(Self(true)),
            "no" => Ok(Self(false)),
            _ => Err(rustical_xml::XmlError::InvalidVariant(format!(
                "Invalid negate-condition parameter: {val}"
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum MatchType {
    Equals,
    #[default]
    Contains,
    StartsWith,
    EndsWith,
}

impl MatchType {
    pub fn match_text(&self, collation: &TextCollation, needle: &str, haystack: &str) -> bool {
        let haystack = collation.normalise(haystack);
        let needle = collation.normalise(needle);

        match &self {
            Self::Equals => haystack == needle,
            Self::Contains => haystack.contains(needle.as_ref()),
            Self::StartsWith => haystack.starts_with(needle.as_ref()),
            Self::EndsWith => haystack.ends_with(needle.as_ref()),
        }
    }
}

impl ValueDeserialize for MatchType {
    fn deserialize(val: &str) -> Result<Self, rustical_xml::XmlError> {
        Ok(match val {
            "equals" => Self::Equals,
            "contains" => Self::Contains,
            "starts-with" => Self::StartsWith,
            "ends-with" => Self::EndsWith,
            _ => {
                return Err(rustical_xml::XmlError::InvalidVariant(format!(
                    "Invalid match-type parameter: {val}"
                )));
            }
        })
    }
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct TextMatchElement {
    #[xml(ty = "attr", default = "Default::default")]
    pub collation: TextCollation,
    #[xml(ty = "attr", default = "Default::default")]
    pub negate_condition: NegateCondition,
    #[xml(ty = "attr", default = "Default::default")]
    pub match_type: MatchType,
    #[xml(ty = "text")]
    pub needle: String,
}

impl TextMatchElement {
    #[must_use]
    pub fn match_property(&self, property: &Property) -> bool {
        let Self {
            collation,
            negate_condition,
            needle,
            match_type,
        } = self;

        let matches = property
            .value
            .as_ref()
            .is_some_and(|haystack| match_type.match_text(collation, needle, haystack));

        // XOR
        negate_condition.0 ^ matches
    }
}

#[cfg(test)]
mod tests {
    use crate::xml::MatchType;

    use super::TextCollation;

    #[test]
    fn test_collation() {
        assert!(!MatchType::Contains.match_text(&TextCollation::AsciiCasemap, "GrÜN", "grünsd"));
        assert!(MatchType::Contains.match_text(&TextCollation::AsciiCasemap, "GrüN", "grün"));
        assert!(!MatchType::Contains.match_text(&TextCollation::Octet, "GrüN", "grün"));
        assert!(MatchType::Contains.match_text(&TextCollation::UnicodeCasemap, "GrÜN", "grün"));
        assert!(MatchType::Contains.match_text(&TextCollation::AsciiCasemap, "GrüN", "grün"));
        assert!(MatchType::Contains.match_text(&TextCollation::AsciiCasemap, "GrüN", "grün"));
        assert!(MatchType::StartsWith.match_text(&TextCollation::Octet, "hello", "hello you"));
        assert!(MatchType::EndsWith.match_text(&TextCollation::Octet, "joe", "joe mama"));
        assert!(MatchType::Equals.match_text(&TextCollation::UnicodeCasemap, "GrÜN", "grün"));
    }
}
