use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarComponent {
    #[serde(rename = "@name")]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarComponentSet {
    #[serde(rename = "C:comp")]
    pub comp: Vec<SupportedCalendarComponent>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct CalendarData {
    #[serde(rename = "@content-type")]
    content_type: String,
    #[serde(rename = "@version")]
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

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarData {
    #[serde(rename = "C:calendar-data", alias = "calendar-data")]
    calendar_data: CalendarData,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ReportMethod {
    CalendarQuery,
    CalendarMultiget,
    SyncCollection,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ReportWrapper {
    #[serde(rename = "$value")]
    report: ReportMethod,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedReportWrapper {
    report: ReportWrapper,
}

impl From<ReportMethod> for SupportedReportWrapper {
    fn from(value: ReportMethod) -> Self {
        Self {
            report: ReportWrapper { report: value },
        }
    }
}

// RFC 3253 section-3.1.5
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedReportSet {
    supported_report: Vec<SupportedReportWrapper>,
}

impl Default for SupportedReportSet {
    fn default() -> Self {
        Self {
            supported_report: vec![
                ReportMethod::CalendarQuery.into(),
                ReportMethod::CalendarMultiget.into(),
                ReportMethod::SyncCollection.into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Transport {
    #[serde(rename = "P:web-push")]
    WebPush,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TransportWrapper {
    #[serde(rename = "$value")]
    transport: Transport,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Transports {
    // NOTE: Here we implement an older version of the spec since the new property name is not reflected
    // in DAVx5 yet
    // https://github.com/bitfireAT/webdav-push/commit/461259a2f2174454b2b00033419b11fac52b79e3
    #[serde(rename = "P:transport")]
    transports: Vec<TransportWrapper>,
}

impl Default for Transports {
    fn default() -> Self {
        Self {
            transports: vec![TransportWrapper {
                transport: Transport::WebPush,
            }],
        }
    }
}
