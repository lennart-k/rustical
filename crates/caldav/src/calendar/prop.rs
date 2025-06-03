use derive_more::derive::{From, Into};
use rustical_ical::CalendarObjectType;
use rustical_xml::{XmlDeserialize, XmlSerialize};

#[derive(Debug, Clone, XmlSerialize, XmlDeserialize, PartialEq, From, Into)]
pub struct SupportedCalendarComponent {
    #[xml(ty = "attr")]
    pub name: CalendarObjectType,
}

#[derive(Debug, Clone, XmlSerialize, XmlDeserialize, PartialEq)]
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

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
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

#[derive(Debug, Clone, XmlSerialize, Default, PartialEq)]
pub struct SupportedCalendarData {
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    calendar_data: CalendarData,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub enum ReportMethod {
    CalendarQuery,
    CalendarMultiget,
    SyncCollection,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct ReportWrapper {
    report: ReportMethod,
}

// RFC 3253 section-3.1.5
#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct SupportedReportSet {
    #[xml(flatten)]
    supported_report: Vec<ReportWrapper>,
}

impl Default for SupportedReportSet {
    fn default() -> Self {
        Self {
            supported_report: vec![
                ReportWrapper {
                    report: ReportMethod::CalendarQuery,
                },
                ReportWrapper {
                    report: ReportMethod::CalendarMultiget,
                },
                ReportWrapper {
                    report: ReportMethod::SyncCollection,
                },
            ],
        }
    }
}
