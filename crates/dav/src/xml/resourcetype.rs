use std::collections::HashMap;

use quick_xml::events::attributes::Attribute;
use quick_xml::events::{BytesEnd, BytesStart, Event};
use quick_xml::name::Namespace;
use rustical_xml::XmlSerialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Resourcetype(pub &'static [&'static str]);

impl XmlSerialize for Resourcetype {
    fn serialize<W: std::io::Write>(
        &self,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()> {
        let tag_str = tag.map(String::from_utf8_lossy);
        if let Some(tag) = &tag_str {
            writer.write_event(Event::Start(BytesStart::new(tag.clone())))?;
        }

        for &ty in self.0 {
            writer.write_event(Event::Empty(BytesStart::new(ty)))?;
        }

        if let Some(tag) = &tag_str {
            writer.write_event(Event::End(BytesEnd::new(tag.clone())))?;
        }
        Ok(())
    }

    #[allow(refining_impl_trait)]
    fn attributes<'a>(&self) -> Option<Vec<Attribute<'a>>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Resourcetype;
    use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

    #[derive(XmlSerialize, XmlRootTag)]
    #[xml(root = b"document")]
    struct Document {
        resourcetype: Resourcetype,
    }

    #[test]
    fn test_serialize_resourcetype() {
        let mut buf = Vec::new();
        let mut writer = quick_xml::Writer::new(&mut buf);
        Document {
            resourcetype: Resourcetype(&["collection", "hello"]),
        }
        .serialize_root(&mut writer)
        .unwrap();
        let out = String::from_utf8(buf).unwrap();
        assert_eq!(
            out,
            "<document><resourcetype><collection/><hello/></resourcetype></document>"
        )
    }
}
