use quick_xml::events::attributes::Attribute;
pub use xml_derive::XmlSerialize;

use crate::XmlRootTag;

pub trait XmlSerialize {
    fn serialize<W: std::io::Write>(
        &self,
        ns: Option<&[u8]>,
        tag: Option<&[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()>;

    fn attributes<'a>(&self) -> impl IntoIterator<Item: Into<Attribute<'a>>>;
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
        self.serialize(Self::root_ns(), Some(Self::root_tag()), writer)
    }
}
