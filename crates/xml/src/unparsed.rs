use std::io::BufRead;

use quick_xml::events::BytesStart;

use crate::{XmlDeserialize, XmlError};

// TODO: actually implement
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unparsed(BytesStart<'static>);

impl Unparsed {
    #[must_use]
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
    ) -> Result<Self, XmlError> {
        // let reader_cloned = NsReader::from_reader(reader.get_ref().to_owned());
        if !empty {
            let mut buf = vec![];
            reader.read_to_end_into(start.name(), &mut buf)?;
        }
        Ok(Self(start.to_owned()))
    }
}
