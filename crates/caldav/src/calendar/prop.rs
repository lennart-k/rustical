use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarComponent {
    #[serde(rename = "@name")]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarComponentSet {
    #[serde(rename = "C:comp")]
    pub comp: Vec<SupportedCalendarComponent>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarData {
    #[serde(rename = "C:calendar-data", alias = "calendar-data")]
    calendar_data: CalendarData,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    #[serde(rename = "C:calendar", alias = "calendar")]
    calendar: (),
    collection: (),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UserPrivilege {
    Read,
    ReadAcl,
    Write,
    WriteAcl,
    WriteContent,
    ReadCurrentUserPrivilegeSet,
    Bind,
    Unbind,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct UserPrivilegeWrapper {
    #[serde(rename = "$value")]
    privilege: UserPrivilege,
}

impl From<UserPrivilege> for UserPrivilegeWrapper {
    fn from(value: UserPrivilege) -> Self {
        Self { privilege: value }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct UserPrivilegeSet {
    privilege: Vec<UserPrivilegeWrapper>,
}

impl Default for UserPrivilegeSet {
    fn default() -> Self {
        Self {
            privilege: vec![
                UserPrivilege::Read.into(),
                UserPrivilege::ReadAcl.into(),
                UserPrivilege::Write.into(),
                UserPrivilege::WriteAcl.into(),
                UserPrivilege::WriteContent.into(),
                UserPrivilege::ReadCurrentUserPrivilegeSet.into(),
                UserPrivilege::Bind.into(),
                UserPrivilege::Unbind.into(),
            ],
        }
    }
}
