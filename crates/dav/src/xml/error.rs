use rustical_xml::{XmlRootTag, XmlSerialize};

#[derive(XmlSerialize, XmlRootTag)]
#[xml(ns = "crate::namespace::NS_DAV", root = b"error")]
#[xml(ns_prefix(
    crate::namespace::NS_DAV = "",
    crate::namespace::NS_CARDDAV = "CARD",
    crate::namespace::NS_CALDAV = "CAL",
    crate::namespace::NS_CALENDARSERVER = "CS",
    crate::namespace::NS_DAVPUSH = "PUSH"
))]
pub struct ErrorElement<'t, T: XmlSerialize>(#[xml(ty = "untagged")] pub &'t T);
