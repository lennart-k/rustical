use derive_more::derive::From;
use quick_xml::{
    events::{BytesEnd, BytesStart, Event},
    name::Namespace,
};
use rustical_xml::{NamespaceOwned, XmlSerialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, From)]
pub struct TagList(Vec<(Option<NamespaceOwned>, String)>);

impl XmlSerialize for TagList {
    fn serialize(
        &self,
        ns: Option<Namespace>,
        tag: Option<&str>,
        namespaces: &HashMap<Namespace, &str>,
        writer: &mut quick_xml::Writer<&mut Vec<u8>>,
    ) -> std::io::Result<()> {
        let prefix = ns.and_then(|ns| namespaces.get(&ns)).map(|prefix| {
            if prefix.is_empty() {
                String::new()
            } else {
                format!("{prefix}:")
            }
        });
        let has_prefix = prefix.is_some();
        let tagname = tag.map(|tag| [&prefix.unwrap_or_default(), tag].concat());

        if let Some(tagname) = tagname.as_ref() {
            let mut bytes_start = BytesStart::new(tagname);
            if !has_prefix && let Some(ns) = &ns {
                bytes_start.push_attribute((b"xmlns".as_ref(), ns.as_ref()));
            }
            writer.write_event(Event::Start(bytes_start))?;
        }

        for (ns, tag) in &self.0 {
            let mut el = writer.create_element(tag);
            if let Some(ns) = ns {
                el = el.with_attribute(("xmlns", String::from_utf8_lossy(&ns.0)));
            }
            el.write_empty()?;
        }

        if let Some(tagname) = tagname.as_ref() {
            writer.write_event(Event::End(BytesEnd::new(tagname)))?;
        }
        Ok(())
    }

    fn attributes<'a>(&self) -> Option<Vec<quick_xml::events::attributes::Attribute<'a>>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::TagList;
    use crate::namespace::NS_DAV;
    use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

    #[derive(XmlSerialize, XmlRootTag)]
    #[xml(root = "document")]
    struct Document {
        taglist: TagList,
    }

    #[test]
    fn test_serialize_taglist() {
        let out = Document {
            taglist: TagList(vec![
                (Some(NS_DAV.into()), "resourcetype".to_owned()),
                (None, "hello".to_owned()),
            ]),
        }
        .serialize_to_string()
        .unwrap();
        assert_eq!(
            out,
            r#"<?xml version="1.0" encoding="utf-8"?>
<document>
    <taglist>
        <resourcetype xmlns="DAV:"/>
        <hello/>
    </taglist>
</document>"#
        );
    }
}
