use derive_more::derive::From;
use quick_xml::{
    events::{BytesEnd, BytesStart, Event},
    name::Namespace,
};
use rustical_xml::{NamespaceOwned, XmlSerialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, From)]
pub struct TagList(Vec<(Option<NamespaceOwned>, String)>);

impl XmlSerialize for TagList {
    fn serialize(
        &self,
        ns: Option<Namespace>,
        tag: Option<&str>,
        namespaces: &HashMap<Namespace, &str>,
        writer: &mut quick_xml::Writer<&mut Vec<u8>>,
    ) -> std::io::Result<()> {
        let prefix = ns
            .map(|ns| namespaces.get(&ns))
            .unwrap_or(None)
            .map(|prefix| {
                if !prefix.is_empty() {
                    format!("{prefix}:")
                } else {
                    String::new()
                }
            });
        let has_prefix = prefix.is_some();
        let tagname = tag.map(|tag| [&prefix.unwrap_or_default(), tag].concat());
        let qname = tagname
            .as_ref()
            .map(|tagname| ::quick_xml::name::QName(tagname.as_bytes()));

        if let Some(qname) = &qname {
            let mut bytes_start = BytesStart::from(qname.to_owned());
            if !has_prefix {
                if let Some(ns) = &ns {
                    bytes_start.push_attribute((b"xmlns".as_ref(), ns.as_ref()));
                }
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

        if let Some(qname) = &qname {
            writer.write_event(Event::End(BytesEnd::from(qname.to_owned())))?;
        }
        Ok(())
    }

    fn attributes<'a>(&self) -> Option<Vec<quick_xml::events::attributes::Attribute<'a>>> {
        None
    }
}
