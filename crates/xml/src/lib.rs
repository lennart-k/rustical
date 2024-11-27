use quick_xml::events::{BytesStart, Event};
use std::io::BufRead;

pub mod de;

pub use de::XmlDeError;
pub use de::XmlDeserialize;
pub use de::XmlRoot;

impl<T: XmlDeserialize> XmlDeserialize for Option<T> {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlDeError> {
        Ok(Some(T::deserialize(reader, start, empty)?))
    }
}

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

impl XmlDeserialize for String {
    fn deserialize<R: BufRead>(
        reader: &mut quick_xml::NsReader<R>,
        start: &BytesStart,
        empty: bool,
    ) -> Result<Self, XmlDeError> {
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
                Event::Eof => return Err(XmlDeError::Eof),
                Event::Text(text) => {
                    content.push_str(&text.unescape()?);
                }
                _a => return Err(XmlDeError::UnknownError),
            };
        }
        Ok(content)
    }
}
