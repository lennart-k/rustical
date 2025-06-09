use rustical_dav::extensions::CommonPropertiesProp;
use rustical_ical::UtcDateTime;
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "CalendarObjectPropName")]
pub enum CalendarObjectProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Getetag(String),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", skip_deserializing)]
    Getcontenttype(&'static str),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    #[xml(prop = "CalendarData")]
    CalendarData(String),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "CalendarObjectPropWrapperName", untagged)]
pub enum CalendarObjectPropWrapper {
    CalendarObject(CalendarObjectProp),
    Common(CommonPropertiesProp),
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ExpandElement {
    #[xml(ty = "attr")]
    pub(crate) start: UtcDateTime,
    #[xml(ty = "attr")]
    pub(crate) end: UtcDateTime,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq, Default, Eq, Hash)]
pub struct CalendarData {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) comp: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) expand: Option<ExpandElement>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) limit_recurrence_set: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    pub(crate) limit_freebusy_set: Option<()>,
}
