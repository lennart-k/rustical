use quick_xml::events::attributes::Attribute;

// An enum keeping track of the XML namespaces used for WebDAV and its extensions
//
// Can also generate appropriate attributes for quick_xml
pub enum Namespace {
    Dav,
    DavPush,
    CalDAV,
    CardDAV,
    ICal,
    CServer,
    Nextcloud,
}

impl Namespace {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dav => "DAV:",
            Self::DavPush => "DAV:Push",
            Self::CalDAV => "urn:ietf:params:xml:ns:caldav",
            Self::CardDAV => "urn:ietf:params:xml:ns:carddav",
            Self::ICal => "http://apple.com/ns/ical/",
            Self::CServer => "http://calendarserver.org/ns/",
            Self::Nextcloud => "http://nextcloud.com/ns",
        }
    }

    // Returns an opinionated namespace attribute name
    pub fn xml_attr(&self) -> &'static str {
        match self {
            Self::Dav => "xmlns",
            Self::DavPush => "xmlns:P",
            Self::CalDAV => "xmlns:C",
            Self::CardDAV => "xmlns:CARD",
            Self::ICal => "xmlns:IC",
            Self::CServer => "xmlns:CS",
            Self::Nextcloud => "xmlns:NEXTC",
        }
    }
}

impl From<Namespace> for Attribute<'static> {
    fn from(value: Namespace) -> Self {
        (value.xml_attr(), value.as_str()).into()
    }
}
