use rustical_xml::{XmlRootTag, XmlSerialize};

#[derive(XmlSerialize, XmlRootTag)]
#[xml(ns = "crate::namespace::NS_DAV", root = b"error")]
#[xml(ns_prefix(
    crate::namespace::NS_DAV = b"",
    crate::namespace::NS_CARDDAV = b"CARD",
    crate::namespace::NS_CALDAV = b"CAL",
    crate::namespace::NS_CALENDARSERVER = b"CS",
    crate::namespace::NS_DAVPUSH = b"PUSH"
))]
pub struct ErrorElement<'t, T: XmlSerialize>(#[xml(ty = "untagged")] pub &'t T);
