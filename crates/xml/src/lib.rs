use quick_xml::events::{BytesStart, Event};
use std::io::BufRead;

pub mod de;
mod error;
pub mod se;
mod value;

pub use de::XmlDeserialize;
pub use de::XmlDocument;
pub use error::XmlDeError;
pub use se::XmlSerialize;
pub use se::XmlSerializeRoot;
pub use value::Value;
pub use xml_derive::XmlRootTag;

impl XmlDeserialize for () {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlDeError> {
        if empty {
            return Ok(());
        }
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if e.name() == start.name() => return Ok(()),
                Event::Eof => return Err(XmlDeError::Eof),
                _ => {}
            };
        }
    }
}

// TODO: actually implement
#[derive(Debug, Clone, PartialEq)]
pub struct Unparsed(BytesStart<'static>);

impl Unparsed {
    pub fn tag_name(&self) -> String {
        // TODO: respect namespace?
        String::from_utf8_lossy(self.0.local_name().as_ref()).to_string()
    }
}

impl XmlDeserialize for Unparsed {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlDeError> {
        // let reader_cloned = NsReader::from_reader(reader.get_ref().to_owned());
        if !empty {
            let mut buf = vec![];
            reader.read_to_end_into(start.name(), &mut buf)?;
        }
        Ok(Self(start.to_owned()))
    }
}

pub trait XmlRootTag {
    fn root_tag() -> &'static [u8];
    fn root_ns() -> Option<&'static [u8]>;
}
