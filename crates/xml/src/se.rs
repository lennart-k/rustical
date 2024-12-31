use std::collections::HashMap;

use quick_xml::{events::attributes::Attribute, name::Namespace};
pub use xml_derive::XmlSerialize;

use crate::XmlRootTag;

pub trait XmlSerialize {
    fn serialize<W: std::io::Write>(
        &self,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()>;

    fn attributes<'a>(&self) -> Option<impl IntoIterator<Item: Into<Attribute<'a>>>>;
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

    #[allow(refining_impl_trait)]
    fn attributes<'a>(&self) -> Option<Vec<Attribute<'a>>> {
        None
    }
}

pub trait XmlSerializeRoot {
    fn serialize_root<W: std::io::Write>(
        &self,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()>;
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
