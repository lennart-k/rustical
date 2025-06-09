use rustical_dav::{extensions::CommonPropertiesProp, xml::HrefElement};
use rustical_store::auth::user::PrincipalType;
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "PrincipalPropName")]
pub enum PrincipalProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(String),

    // Scheduling Extensions to CalDAV (RFC 6638)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", skip_deserializing)]
    CalendarUserType(PrincipalType),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarUserAddressSet(HrefElement),

    // WebDAV Access Control (RFC 3744)
    #[xml(ns = "rustical_dav::namespace::NS_DAV", rename = b"principal-URL")]
    PrincipalUrl(HrefElement),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarHomeSet(CalendarHomeSet),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "PrincipalPropWrapperName", untagged)]
pub enum PrincipalPropWrapper {
    Principal(PrincipalProp),
    Common(CommonPropertiesProp),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone)]
pub struct CalendarHomeSet(#[xml(ty = "untagged", flatten)] pub(super) Vec<HrefElement>);
