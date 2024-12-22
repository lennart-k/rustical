use std::io::BufRead;
pub use xml_derive::XmlDeserialize;
pub use xml_derive::XmlRoot;

use quick_xml::events::{BytesStart, Event};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum XmlDeError {
    #[error(transparent)]
    QuickXmlDeError(#[from] quick_xml::de::DeError),
    #[error(transparent)]
    QuickXmlError(#[from] quick_xml::Error),
    #[error(transparent)]
    QuickXmlAttrError(#[from] quick_xml::events::attributes::AttrError),
    #[error("Unknown error")]
    UnknownError,
    #[error("Invalid tag {0}. Expected {1}")]
    InvalidTag(String, String),
    #[error("Missing field {0}")]
    MissingField(&'static str),
    #[error("End of file, expected closing tags")]
    Eof,
    #[error("Unsupported xml event: {0}")]
    UnsupportedEvent(&'static str),
    #[error("{0}")]
    Other(String),
    #[error("Invalid variant: {0}")]
    InvalidVariant(String),
    #[error("Invalid field name: ")]
    InvalidFieldName,
    #[error(transparent)]
    InvalidValue(#[from] crate::value::ParseValueError),
}

pub trait XmlDeserialize: Sized {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlDeError>;
}

pub trait XmlRoot {
    fn parse<R: BufRead>(mut reader: quick_xml::NsReader<R>) -> Result<Self, XmlDeError>
    where
        Self: XmlDeserialize,
    {
        let mut buf = Vec::new();
        let event = reader.read_event_into(&mut buf)?;
        let empty = event.is_empty();
        match event {
            Event::Start(start) | Event::Empty(start) => {
                let (_ns, name) = reader.resolve_element(start.name());
                if name.as_ref() != Self::root_tag() {
                    return Err(XmlDeError::InvalidTag(
                        String::from_utf8_lossy(name.as_ref()).to_string(),
                        String::from_utf8_lossy(Self::root_tag()).to_string(),
                    ));
                };

                // TODO: check namespace

                return Self::deserialize(&mut reader, &start, empty);
            }
            _ => {}
        };
        Err(XmlDeError::UnknownError)
    }

    fn parse_reader<R: BufRead>(input: R) -> Result<Self, XmlDeError>
    where
        Self: XmlDeserialize,
    {
        let mut reader = quick_xml::NsReader::from_reader(input);
        reader.config_mut().trim_text(true);
        Self::parse(reader)
    }

    fn root_tag() -> &'static [u8];
}

pub trait XmlRootParseStr<'i>: XmlRoot + XmlDeserialize {
    #[inline]
    fn parse_str(s: &'i str) -> Result<Self, XmlDeError> {
        Self::parse_reader(s.as_bytes())
    }
}

impl<T: XmlRoot + XmlDeserialize> XmlRootParseStr<'_> for T {}
