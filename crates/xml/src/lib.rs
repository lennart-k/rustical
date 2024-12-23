use quick_xml::events::{BytesStart, Event};
use std::io::BufRead;

pub mod de;
pub mod se;
mod value;

pub use de::XmlDeError;
pub use de::XmlDeserialize;
pub use de::XmlDocument;
pub use de::XmlRootTag;
pub use se::XmlSerialize;
pub use value::Value;

// impl<T: XmlDeserialize> XmlDeserialize for Option<T> {
//     fn deserialize<R: BufRead>(
//         reader: &mut quick_xml::NsReader<R>,
//         start: &BytesStart,
//         empty: bool,
//     ) -> Result<Self, XmlDeError> {
//         Ok(Some(T::deserialize(reader, start, empty)?))
//     }
// }

#[derive(Debug, Clone, PartialEq)]
pub struct Unit;

impl XmlDeserialize for Unit {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlDeError> {
        if empty {
            return Ok(Unit);
        }
        let mut buf = Vec::new();
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::End(e) if e.name() == start.name() => return Ok(Unit),
                Event::Eof => return Err(XmlDeError::Eof),
                _ => {}
            };
        }
    }
}

// TODO: actually implement
#[derive(Debug, Clone, PartialEq)]
pub struct Unparsed(BytesStart<'static>);

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
