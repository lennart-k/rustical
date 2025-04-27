use rustical_xml::XmlDeserialize;
use rustical_xml::XmlRootTag;

#[derive(Debug, Clone, XmlDeserialize, XmlRootTag, PartialEq)]
#[xml(root = b"propfind", ns = "crate::namespace::NS_DAV")]
pub struct PropfindElement {
    #[xml(ty = "untagged")]
    pub prop: PropfindType,
}

#[derive(Debug, Clone, XmlDeserialize, PartialEq)]
pub struct PropElement<PN: XmlDeserialize = Propname>(#[xml(ty = "untagged", flatten)] pub Vec<PN>);

#[derive(Debug, Clone, XmlDeserialize, PartialEq)]
pub struct Propname(#[xml(ty = "tag_name")] pub String);

#[derive(Debug, Clone, XmlDeserialize, PartialEq)]
pub enum PropfindType<PN: XmlDeserialize = Propname> {
    #[xml(ns = "crate::namespace::NS_DAV")]
    Propname,
    #[xml(ns = "crate::namespace::NS_DAV")]
    Allprop,
    #[xml(ns = "crate::namespace::NS_DAV")]
    Prop(PropElement<PN>),
}
