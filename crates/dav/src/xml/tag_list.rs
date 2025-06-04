use derive_more::derive::From;
use quick_xml::{
    events::{BytesStart, Event},
    name::Namespace,
};
use rustical_xml::{NamespaceOwned, XmlSerialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, From)]
pub struct TagList(Vec<(Option<NamespaceOwned>, String)>);

impl XmlSerialize for TagList {
    fn serialize<W: std::io::Write>(
        &self,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()> {
        for (_ns, tag) in &self.0 {
            writer.write_event(Event::Empty(BytesStart::new(tag)))?;
        }
        Ok(())
    }

    #[allow(refining_impl_trait)]
    fn attributes<'a>(&self) -> Option<Vec<quick_xml::events::attributes::Attribute<'a>>> {
        None
    }
}
