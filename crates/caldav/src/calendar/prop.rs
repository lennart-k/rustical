use derive_more::derive::{From, Into};
use rustical_ical::CalendarObjectType;
use rustical_xml::{XmlDeserialize, XmlSerialize};
use strum_macros::VariantArray;

#[derive(Debug, Clone, XmlSerialize, XmlDeserialize, PartialEq, Eq, From, Into)]
pub struct SupportedCalendarComponent {
    #[xml(ty = "attr")]
    pub name: CalendarObjectType,
}

#[derive(Debug, Clone, XmlSerialize, XmlDeserialize, PartialEq, Eq)]
pub struct SupportedCalendarComponentSet {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", flatten)]
    pub comp: Vec<SupportedCalendarComponent>,
}

impl From<Vec<CalendarObjectType>> for SupportedCalendarComponentSet {
    fn from(value: Vec<CalendarObjectType>) -> Self {
        Self {
            comp: value
                .into_iter()
                .map(SupportedCalendarComponent::from)
                .collect(),
        }
    }
}

impl From<SupportedCalendarComponentSet> for Vec<CalendarObjectType> {
    fn from(value: SupportedCalendarComponentSet) -> Self {
        value
            .comp
            .into_iter()
            .map(CalendarObjectType::from)
            .collect()
    }
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct CalendarData {
    #[xml(ty = "attr")]
    content_type: String,
    #[xml(ty = "attr")]
    version: String,
}

impl Default for CalendarData {
    fn default() -> Self {
        Self {
            content_type: "text/calendar".to_owned(),
            version: "2.0".to_owned(),
        }
    }
}

#[derive(Debug, Clone, XmlSerialize, Default, PartialEq, Eq)]
pub struct SupportedCalendarData {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    calendar_data: CalendarData,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq, VariantArray)]
pub enum ReportMethod {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarQuery,
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarMultiget,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SyncCollection,
}
