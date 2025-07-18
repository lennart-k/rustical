use rustical_dav::{
    extensions::CommonPropertiesProp,
    xml::{GroupMemberSet, GroupMembership, HrefElement},
};
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "PrincipalPropName")]
pub enum PrincipalProp {
    // WebDAV Access Control (RFC 3744)
    #[xml(rename = b"principal-URL")]
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    PrincipalUrl(HrefElement),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    GroupMembership(GroupMembership),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    GroupMemberSet(GroupMemberSet),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", rename = b"alternate-URI-set")]
    AlternateUriSet,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    PrincipalCollectionSet(HrefElement),

    // CardDAV (RFC 6352)
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookHomeSet(AddressbookHomeSet),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    PrincipalAddress(Option<HrefElement>),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone)]
pub struct AddressbookHomeSet(#[xml(ty = "untagged", flatten)] pub Vec<HrefElement>);

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "PrincipalPropWrapperName", untagged)]
pub enum PrincipalPropWrapper {
    Principal(PrincipalProp),
    Common(CommonPropertiesProp),
}
