pub use xml_derive::XmlSerialize;

pub trait XmlSerialize {
    fn serialize<W: std::io::Write>(
        &self,
        tag: Option<&[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()>;
}
