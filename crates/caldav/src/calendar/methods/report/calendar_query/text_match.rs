use ical::property::Property;
use rustical_xml::{ValueDeserialize, XmlDeserialize};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum TextCollation {
    #[default]
    AsciiCasemap,
    Octet,
}

impl TextCollation {
    // Check whether a haystack contains a needle respecting the collation
    pub fn match_text(&self, needle: &str, haystack: &str) -> bool {
        match self {
            // https://datatracker.ietf.org/doc/html/rfc4790#section-9.2
            Self::AsciiCasemap => haystack
                .to_ascii_uppercase()
                .contains(&needle.to_ascii_uppercase()),
            Self::Octet => haystack.contains(needle),
        }
    }
}

impl ValueDeserialize for TextCollation {
    fn deserialize(val: &str) -> Result<Self, rustical_xml::XmlError> {
        match val {
            "i;ascii-casemap" => Ok(Self::AsciiCasemap),
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

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct TextMatchElement {
    #[xml(ty = "attr", default = "Default::default")]
    pub collation: TextCollation,
    #[xml(ty = "attr", default = "Default::default")]
    pub(crate) negate_condition: NegateCondition,
    #[xml(ty = "text")]
    pub(crate) needle: String,
}

impl TextMatchElement {
    pub fn match_property(&self, property: &Property) -> bool {
        let Self {
            collation,
            negate_condition,
            needle,
        } = self;

        let matches = property
            .value
            .as_ref()
            .is_some_and(|haystack| collation.match_text(needle, haystack));

        // XOR
        negate_condition.0 ^ matches
    }
}

#[cfg(test)]
mod tests {
    use crate::calendar::methods::report::calendar_query::text_match::TextCollation;

    #[test]
    fn test_collation() {
        assert!(TextCollation::AsciiCasemap.match_text("GrüN", "grün"));
        assert!(!TextCollation::AsciiCasemap.match_text("GrÜN", "grün"));
        assert!(!TextCollation::Octet.match_text("GrÜN", "grün"));
        assert!(TextCollation::Octet.match_text("hallo", "hallo"));
        assert!(TextCollation::AsciiCasemap.match_text("HaLlo", "hAllo"));
    }
}
