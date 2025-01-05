use derive_more::derive::From;
use quick_xml::name::Namespace;
use rustical_xml::XmlSerialize;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, From)]
pub struct TagList(Vec<String>);

impl XmlSerialize for TagList {
    fn serialize<W: std::io::Write>(
        &self,
        ns: Option<Namespace>,
        tag: Option<&[u8]>,
        namespaces: &HashMap<Namespace, &[u8]>,
        writer: &mut quick_xml::Writer<W>,
    ) -> std::io::Result<()> {
        #[derive(Debug, XmlSerialize, PartialEq)]
        struct Inner(#[xml(ty = "untagged", flatten)] Vec<Tag>);

        #[derive(Debug, XmlSerialize, PartialEq)]
        struct Tag(#[xml(ty = "tag_name")] String);
        Inner(self.0.iter().map(|t| Tag(t.to_owned())).collect())
            .serialize(ns, tag, namespaces, writer)
    }

    #[allow(refining_impl_trait)]
    fn attributes<'a>(&self) -> Option<Vec<quick_xml::events::attributes::Attribute<'a>>> {
        None
    }
}
