use rustical_xml::XmlDeserialize;
use rustical_xml::XmlRootTag;

#[derive(Debug, Clone, XmlDeserialize, XmlRootTag, PartialEq)]
#[xml(root = b"propfind", ns = "crate::namespace::NS_DAV")]
pub struct PropfindElement {
    #[xml(ns = "crate::namespace::NS_DAV")]
    pub prop: PropfindType,
}

#[derive(Debug, Clone, XmlDeserialize, PartialEq)]
pub struct PropElement(#[xml(ty = "untagged", flatten)] pub Vec<Propname>);

#[derive(Debug, Clone, XmlDeserialize, PartialEq)]
pub struct Propname(#[xml(ty = "tag_name")] pub String);

#[derive(Debug, Clone, XmlDeserialize, PartialEq)]
pub enum PropfindType {
    Propname,
    Allprop,
    Prop(PropElement),
}
