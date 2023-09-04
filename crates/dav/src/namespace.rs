use quick_xml::events::attributes::Attribute;

// An enum keeping track of the XML namespaces used for WebDAV and its extensions
//
// Can also generate appropriate attributes for quick_xml
pub enum Namespace {
    Dav,
    CalDAV,
    ICal,
    CServer,
}

impl Namespace {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dav => "DAV:",
            Self::CalDAV => "urn:ietf:params:xml:ns:caldav",
            Self::ICal => "http://apple.com/ns/ical/",
            Self::CServer => "http://calendarserver.org/ns/",
        }
    }

    // Returns an opinionated namespace attribute name
    pub fn xml_attr(&self) -> &'static str {
        match self {
            Self::Dav => "xmlns",
            Self::CalDAV => "xmlns:C",
            Self::ICal => "xmlns:IC",
            Self::CServer => "xmlns:CS",
        }
    }
}

impl From<Namespace> for Attribute<'static> {
    fn from(value: Namespace) -> Self {
        (value.xml_attr(), value.as_str()).into()
    }
}
