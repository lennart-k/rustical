use rustical_dav::{
    extensions::CommonPropertiesProp,
    xml::{GroupMemberSet, GroupMembership, HrefElement, SupportedReportSet},
};
use rustical_store::auth::PrincipalType;
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};
use strum_macros::VariantArray;

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "PrincipalPropName")]
pub enum PrincipalProp {
    // Scheduling Extensions to CalDAV (RFC 6638)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", skip_deserializing)]
    CalendarUserType(PrincipalType),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarUserAddressSet(HrefElement),

    // WebDAV Access Control (RFC 3744)
    #[xml(ns = "rustical_dav::namespace::NS_DAV", rename = "principal-URL")]
    PrincipalUrl(HrefElement),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    GroupMembership(GroupMembership),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    GroupMemberSet(GroupMemberSet),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", rename = "alternate-URI-set")]
    AlternateUriSet,
    // #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    // PrincipalCollectionSet(HrefElement),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", skip_deserializing)]
    SupportedReportSet(SupportedReportSet<ReportMethod>),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarHomeSet(CalendarHomeSet),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone)]
pub struct CalendarHomeSet(#[xml(ty = "untagged", flatten)] pub Vec<HrefElement>);

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "PrincipalPropWrapperName", untagged)]
pub enum PrincipalPropWrapper {
    Principal(PrincipalProp),
    Common(CommonPropertiesProp),
}

#[derive(XmlSerialize, PartialEq, Eq, Clone, VariantArray)]
pub enum ReportMethod {
    // We don't actually support principal-match
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    PrincipalMatch,
}
