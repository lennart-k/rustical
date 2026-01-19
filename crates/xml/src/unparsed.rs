use std::io::BufRead;

use quick_xml::{events::BytesStart, name::ResolveResult};

use crate::{NamespaceOwned, XmlDeserialize, XmlError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unparsed(pub Option<NamespaceOwned>, pub String);

impl Unparsed {
    #[must_use]
    pub const fn ns(&self) -> Option<&NamespaceOwned> {
        self.0.as_ref()
    }

    #[must_use]
    pub const fn tag_name(&self) -> &str {
        self.1.as_str()
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
        let (ns, tag_name) = reader.resolver().resolve_element(start.name());
        let ns: Option<NamespaceOwned> = match ns {
            ResolveResult::Bound(ns) => Some(ns.into()),
            ResolveResult::Unbound | ResolveResult::Unknown(_) => None,
        };
        let tag_name = String::from_utf8_lossy(tag_name.as_ref()).to_string();
        Ok(Self(ns, tag_name))
    }
}
