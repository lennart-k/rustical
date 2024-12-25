use quick_xml::name::Namespace;
use quick_xml::name::ResolveResult;
use std::io::BufRead;
pub use xml_derive::XmlDeserialize;
pub use xml_derive::XmlDocument;
pub use xml_derive::XmlRootTag;

use quick_xml::events::{BytesStart, Event};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum XmlDeError {
    #[error(transparent)]
    QuickXmlError(#[from] quick_xml::Error),
    #[error(transparent)]
    QuickXmlAttrError(#[from] quick_xml::events::attributes::AttrError),
    #[error("Unknown error")]
    UnknownError,
    #[error("Invalid tag [{0}]{1}. Expected [{2}]{3}")]
    InvalidTag(String, String, String, String),
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
    #[error("Invalid field name in {0}: {1}")]
    InvalidFieldName(&'static str, String),
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

pub trait XmlRootTag {
    fn root_tag() -> &'static [u8];
    fn root_ns() -> Option<&'static [u8]>;
}

pub trait XmlDocument: XmlDeserialize {
    fn parse<R: BufRead>(reader: quick_xml::NsReader<R>) -> Result<Self, XmlDeError>;

    #[inline]
    fn parse_reader<R: BufRead>(input: R) -> Result<Self, XmlDeError>
    where
        Self: XmlDeserialize,
    {
        let mut reader = quick_xml::NsReader::from_reader(input);
        reader.config_mut().trim_text(true);
        Self::parse(reader)
    }

    #[inline]
    fn parse_str(s: &str) -> Result<Self, XmlDeError> {
        Self::parse_reader(s.as_bytes())
    }
}

impl<T: XmlRootTag + XmlDeserialize> XmlDocument for T {
    fn parse<R: BufRead>(mut reader: quick_xml::NsReader<R>) -> Result<Self, XmlDeError>
    where
        Self: XmlDeserialize,
    {
        let mut buf = Vec::new();
        loop {
            let event = reader.read_event_into(&mut buf)?;
            let empty = matches!(event, Event::Empty(_));
            match event {
                Event::Decl(_) => { /* <?xml ... ?> ignore this */ }
                Event::Comment(_) => { /*  ignore this */ }
                Event::Start(start) | Event::Empty(start) => {
                    let (ns, name) = reader.resolve_element(start.name());
                    let matches = match (Self::root_ns(), &ns, name) {
                        // Wrong tag
                        (_, _, name) if name.as_ref() != Self::root_tag() => false,
                        // Wrong namespace
                        (Some(root_ns), ns, _)
                            if &ResolveResult::Bound(Namespace(root_ns)) != ns =>
                        {
                            false
                        }
                        _ => true,
                    };
                    if !matches {
                        let root_ns = Self::root_ns();
                        return Err(XmlDeError::InvalidTag(
                            format!("{ns:?}"),
                            String::from_utf8_lossy(name.as_ref()).to_string(),
                            format!("{root_ns:?}"),
                            String::from_utf8_lossy(Self::root_tag()).to_string(),
                        ));
                    };

                    return Self::deserialize(&mut reader, &start, empty);
                }
                _ => return Err(XmlDeError::UnknownError),
            };
        }
    }
}
