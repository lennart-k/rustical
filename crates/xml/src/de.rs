use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use std::io::BufRead;
pub use xml_derive::XmlDeserialize;
pub use xml_derive::XmlDocument;

use crate::XmlError;
use crate::XmlRootTag;

pub trait XmlDeserialize: Sized {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlError>;
}

pub trait XmlDocument: XmlDeserialize {
    fn parse<R: BufRead>(reader: quick_xml::NsReader<R>) -> Result<Self, XmlError>;

    #[inline]
    fn parse_reader<R: BufRead>(input: R) -> Result<Self, XmlError>
    where
        Self: XmlDeserialize,
    {
        let mut reader = quick_xml::NsReader::from_reader(input);
        reader.config_mut().trim_text(true);
        Self::parse(reader)
    }

    #[inline]
    fn parse_str(s: &str) -> Result<Self, XmlError> {
        Self::parse_reader(s.as_bytes())
    }
}

impl<T: XmlRootTag + XmlDeserialize> XmlDocument for T {
    fn parse<R: BufRead>(mut reader: quick_xml::NsReader<R>) -> Result<Self, XmlError>
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
                        (Some(root_ns), ns, _) if &ResolveResult::Bound(root_ns) != ns => false,
                        _ => true,
                    };
                    if !matches {
                        let root_ns = Self::root_ns();
                        return Err(XmlError::InvalidTag(
                            format!("{ns:?}"),
                            String::from_utf8_lossy(name.as_ref()).to_string(),
                            format!("{root_ns:?}"),
                            String::from_utf8_lossy(Self::root_tag()).to_string(),
                        ));
                    };

                    return Self::deserialize(&mut reader, &start, empty);
                }
                _ => return Err(XmlError::UnsupportedEvent("unknown, todo")),
            };
        }
    }
}
