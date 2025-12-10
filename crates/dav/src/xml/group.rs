use crate::xml::HrefElement;
use rustical_xml::{XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, Debug)]
pub struct GroupMembership(#[xml(ty = "untagged", flatten)] pub Vec<HrefElement>);

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, Debug)]
pub struct GroupMemberSet(#[xml(ty = "untagged", flatten)] pub Vec<HrefElement>);
