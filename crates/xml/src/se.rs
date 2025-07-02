use crate::XmlRootTag;
use quick_xml::{
    events::{BytesStart, Event, attributes::Attribute},
    name::{Namespace, QName},
};
use std::collections::HashMap;
pub use xml_derive::XmlSerialize;

pub trait XmlSerialize {
    fn serialize<W: std::io::Write>(
        &self,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()>;

    fn attributes<'a>(&self) -> Option<Vec<Attribute<'a>>>;
}

impl<T: XmlSerialize> XmlSerialize for Option<T> {
    fn serialize<W: std::io::Write>(
        &self,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()> {
        if let Some(some) = self {
            some.serialize(ns, tag, namespaces, writer)
        } else {
            Ok(())
        }
    }

    fn attributes<'a>(&self) -> Option<Vec<Attribute<'a>>> {
        None
    }
}

pub trait XmlSerializeRoot {
    fn serialize_root<W: std::io::Write>(
        &self,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()>;

    fn serialize_to_string(&self) -> std::io::Result<String> {
        let mut buf: Vec<_> = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".into();
        let mut writer = quick_xml::Writer::new(&mut buf);
        self.serialize_root(&mut writer)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }
}

impl<T: XmlSerialize + XmlRootTag> XmlSerializeRoot for T {
    fn serialize_root<W: std::io::Write>(
        &self,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()> {
        let namespaces = Self::root_ns_prefixes();
        self.serialize(Self::root_ns(), Some(Self::root_tag()), &namespaces, writer)
    }
}

impl XmlSerialize for () {
    fn serialize<W: std::io::Write>(
        &self,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()> {
        let prefix = ns
            .map(|ns| namespaces.get(&ns))
            .unwrap_or(None)
            .map(|prefix| {
                if !prefix.is_empty() {
                    [*prefix, b":"].concat()
                } else {
                    Vec::new()
                }
            });
        let has_prefix = prefix.is_some();
        let tagname = tag.map(|tag| [&prefix.unwrap_or_default(), tag].concat());
        let qname = tagname.as_ref().map(|tagname| QName(tagname));
        if let Some(qname) = &qname {
            let mut bytes_start = BytesStart::from(qname.to_owned());
            if !has_prefix {
                if let Some(ns) = &ns {
                    bytes_start.push_attribute((b"xmlns".as_ref(), ns.as_ref()));
                }
            }
            writer.write_event(Event::Empty(bytes_start))?;
        }
        Ok(())
    }

    fn attributes<'a>(&self) -> Option<Vec<quick_xml::events::attributes::Attribute<'a>>> {
        None
    }
}
