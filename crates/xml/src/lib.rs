use quick_xml::events::{BytesStart, Event};
use std::io::BufRead;
use thiserror::Error;
pub use xml_derive::XmlDeserialize;

#[derive(Debug, Error)]
pub enum XmlError {
    #[error(transparent)]
    QuickXmlDeError(#[from] quick_xml::de::DeError),
    #[error(transparent)]
    QuickXmlError(#[from] quick_xml::Error),
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
}

pub trait XmlDeserialize: Sized {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlError>;
}

pub trait XmlRoot: XmlDeserialize {
    fn parse<R: BufRead>(mut reader: quick_xml::NsReader<R>) -> Result<Self, XmlError> {
        let mut buf = Vec::new();
        let event = reader.read_event_into(&mut buf)?;
        match event {
            Event::Start(start) => {
                let (_ns, name) = reader.resolve_element(start.name());
                if name.as_ref() != Self::root_tag() {
                    return Err(XmlError::InvalidTag(
                        String::from_utf8_lossy(name.as_ref()).to_string(),
                        String::from_utf8_lossy(Self::root_tag()).to_string(),
                    ));
                };
                // TODO: check namespace

                return Self::deserialize(&mut reader, &start, false);
            }
            Event::Empty(start) => {
                let (_ns, name) = reader.resolve_element(start.name());
                if name.as_ref() != Self::root_tag() {
                    return Err(XmlError::InvalidTag(
                        String::from_utf8_lossy(name.as_ref()).to_string(),
                        String::from_utf8_lossy(Self::root_tag()).to_string(),
                    ));
                };
                // TODO: check namespace

                return Self::deserialize(&mut reader, &start, true);
            }
            _ => {}
        };
        Err(XmlError::UnknownError)
    }

    fn parse_str(input: &str) -> Result<Self, XmlError> {
        let mut reader = quick_xml::NsReader::from_str(input);
        reader.config_mut().trim_text(true);
        Self::parse(reader)
    }

    fn root_tag() -> &'static [u8];
}

impl<T: XmlDeserialize> XmlDeserialize for Option<T> {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlError> {
        Ok(Some(T::deserialize(reader, start, empty)?))
    }
}

pub struct Unit;

impl XmlDeserialize for Unit {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlError> {
        if empty {
            return Ok(Unit);
        }
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if e.name() == start.name() => return Ok(Unit),
                Event::Eof => return Err(XmlError::Eof),
                _ => {}
            };
        }
    }
}

impl XmlDeserialize for String {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlError> {
        if empty {
            return Ok(String::new());
        }
        let mut content = String::new();
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if e.name() == start.name() => {
                    break;
                }
                Event::Eof => return Err(XmlError::Eof),
                Event::Text(text) => {
                    content.push_str(&text.unescape()?);
                }
                _a => return Err(XmlError::UnknownError),
            };
        }
        Ok(content)
    }
}
