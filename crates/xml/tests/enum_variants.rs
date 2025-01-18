use std::str::FromStr;

use quick_xml::name::Namespace;
use rustical_xml::EnumVariants;
use xml_derive::EnumUnitVariants;

pub const NS_DAV: Namespace = Namespace(b"DAV:");
pub const NS_DAVPUSH: Namespace = Namespace(b"https://bitfire.at/webdav-push");
pub const NS_CALDAV: Namespace = Namespace(b"urn:ietf:params:xml:ns:caldav");
pub const NS_CARDDAV: Namespace = Namespace(b"urn:ietf:params:xml:ns:carddav");
pub const NS_ICAL: Namespace = Namespace(b"http://apple.com/ns/ical/");
pub const NS_CALENDARSERVER: Namespace = Namespace(b"http://calendarserver.org/ns/");
pub const NS_NEXTCLOUD: Namespace = Namespace(b"http://nextcloud.com/ns");

#[derive(EnumVariants)]
enum ExtensionProp {
    Hello,
}

#[derive(EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "CalendarPropName")]
enum CalendarProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "NS_DAV")]
    Displayname(Option<String>),
    #[xml(ns = "NS_DAV")]
    Getcontenttype(&'static str),

    #[xml(ns = "NS_DAV", rename = b"principal-URL")]
    PrincipalUrl,
    Topic,
}

#[test]
fn test_enum_tagged_variants() {
    assert_eq!(
        CalendarProp::TAGGED_VARIANTS,
        &[
            (Some(NS_DAV), "displayname"),
            (Some(NS_DAV), "getcontenttype"),
            (Some(NS_DAV), "principal-URL"),
            (None, "topic"),
        ]
    );
}

#[derive(EnumVariants)]
#[xml(untagged)]
enum UnionProp {
    Calendar(CalendarProp),
    Extension(ExtensionProp),
}

#[test]
fn test_enum_untagged_variants() {
    assert_eq!(
        UnionProp::variant_names(),
        vec![
            (Some(NS_DAV), "displayname"),
            (Some(NS_DAV), "getcontenttype"),
            (Some(NS_DAV), "principal-URL"),
            (None, "topic"),
            (None, "hello"),
        ]
    );
}

#[test]
fn test_enum_unit_variants() {
    let displayname: (Option<Namespace>, &str) = CalendarPropName::Displayname.into();
    assert_eq!(displayname, (Some(NS_DAV), "displayname"));
    let topic: (Option<Namespace>, &str) = CalendarPropName::Topic.into();
    assert_eq!(topic, (None, "topic"));

    let propname: CalendarPropName = CalendarProp::Displayname(None).into();
    let displayname: (Option<Namespace>, &str) = propname.into();
    assert_eq!(displayname, (Some(NS_DAV), "displayname"));

    let propname: CalendarPropName = FromStr::from_str("displayname").unwrap();
    assert_eq!(displayname, (Some(NS_DAV), "displayname"));
}
