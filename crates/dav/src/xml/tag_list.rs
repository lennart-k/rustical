use derive_more::derive::From;
use quick_xml::name::Namespace;
use rustical_xml::{NamespaceOwned, XmlSerialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, From)]
pub struct TagList(Vec<(Option<NamespaceOwned>, String)>);

impl XmlSerialize for TagList {
    fn serialize<W: std::io::Write>(
        &self,
        _ns: Option<Namespace>,
        _tag: Option<&[u8]>,
        _namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()> {
        for (ns, tag) in &self.0 {
            let mut el = writer.create_element(tag);
            if let Some(ns) = ns {
                el = el.with_attribute(("xmlns", String::from_utf8_lossy(&ns.0)));
            }
            el.write_empty()?;
        }
        Ok(())
    }

    #[allow(refining_impl_trait)]
    fn attributes<'a>(&self) -> Option<Vec<quick_xml::events::attributes::Attribute<'a>>> {
        None
    }
}
