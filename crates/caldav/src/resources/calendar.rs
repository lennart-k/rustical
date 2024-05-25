use actix_web::{web::Data, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::error::Error;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml_snippets::{HrefElement, TextNode};
use rustical_store::calendar::{Calendar, CalendarStore};
use serde::Serialize;
use std::sync::Arc;
use strum::{EnumString, IntoStaticStr, VariantNames};
use tokio::sync::RwLock;

pub struct CalendarResource<C: CalendarStore + ?Sized> {
    pub cal_store: Arc<RwLock<C>>,
    pub path: String,
    pub principal: String,
    pub calendar_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarComponent {
    #[serde(rename = "@name")]
    pub name: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarComponentSet {
    #[serde(rename = "C:comp")]
    pub comp: Vec<SupportedCalendarComponent>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct CalendarData {
    #[serde(rename = "@content-type")]
    content_type: &'static str,
    #[serde(rename = "@version")]
    version: &'static str,
}

impl Default for CalendarData {
    fn default() -> Self {
        Self {
            content_type: "text/calendar",
            version: "2.0",
        }
    }
}

#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarData {
    #[serde(rename = "C:calendar-data")]
    calendar_data: CalendarData,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    #[serde(rename = "C:calendar")]
    calendar: (),
    collection: (),
}

#[derive(Serialize)]
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

#[derive(Serialize)]
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

#[derive(Serialize)]
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

#[derive(EnumString, Debug, VariantNames, IntoStaticStr, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum CalendarProp {
    Resourcetype,
    CurrentUserPrincipal,
    Owner,
    Displayname,
    CalendarColor,
    CalendarDescription,
    SupportedCalendarComponentSet,
    SupportedCalendarData,
    Getcontenttype,
    CurrentUserPrivilegeSet,
    MaxResourceSize,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalendarPropResponse {
    Resourcetype(Resourcetype),
    CurrentUserPrincipal(HrefElement),
    Owner(HrefElement),
    Displayname(TextNode),
    #[serde(rename = "IC:calendar-color", alias = "calendar-color")]
    CalendarColor(TextNode),
    #[serde(rename = "C:calendar-description", alias = "calendar-description")]
    CalendarDescription(TextNode),
    #[serde(
        rename = "C:supported-calendar-component-set",
        alias = "supported-calendar-component-set"
    )]
    SupportedCalendarComponentSet(SupportedCalendarComponentSet),
    #[serde(
        rename = "C:supported-calendar-data",
        alias = "supported-calendar-data"
    )]
    SupportedCalendarData(SupportedCalendarData),
    Getcontenttype(TextNode),
    MaxResourceSize(TextNode),
    CurrentUserPrivilegeSet(UserPrivilegeSet),
}

pub struct CalendarFile {
    pub calendar: Calendar,
    pub principal: String,
    pub path: String,
}

impl Resource for CalendarFile {
    type PropType = CalendarProp;
    type PropResponse = CalendarPropResponse;

    fn get_prop(&self, prefix: &str, prop: Self::PropType) -> Result<Self::PropResponse> {
        match prop {
            CalendarProp::Resourcetype => {
                Ok(CalendarPropResponse::Resourcetype(Resourcetype::default()))
            }
            CalendarProp::CurrentUserPrincipal => Ok(CalendarPropResponse::CurrentUserPrincipal(
                HrefElement::new(format!("{}/{}/", prefix, self.principal)),
            )),
            CalendarProp::Owner => Ok(CalendarPropResponse::Owner(HrefElement::new(format!(
                "{}/{}/",
                prefix, self.principal
            )))),
            CalendarProp::Displayname => Ok(CalendarPropResponse::Displayname(TextNode(
                self.calendar.name.clone(),
            ))),
            CalendarProp::CalendarColor => Ok(CalendarPropResponse::CalendarColor(TextNode(
                self.calendar.color.clone(),
            ))),
            CalendarProp::CalendarDescription => Ok(CalendarPropResponse::CalendarDescription(
                TextNode(self.calendar.description.clone()),
            )),
            CalendarProp::SupportedCalendarComponentSet => {
                Ok(CalendarPropResponse::SupportedCalendarComponentSet(
                    SupportedCalendarComponentSet {
                        comp: vec![SupportedCalendarComponent { name: "VEVENT" }],
                    },
                ))
            }
            CalendarProp::SupportedCalendarData => Ok(CalendarPropResponse::SupportedCalendarData(
                SupportedCalendarData::default(),
            )),
            CalendarProp::Getcontenttype => Ok(CalendarPropResponse::Getcontenttype(TextNode(
                Some("text/calendar;charset=utf-8".to_owned()),
            ))),
            CalendarProp::MaxResourceSize => Ok(CalendarPropResponse::MaxResourceSize(TextNode(
                Some("10000000".to_owned()),
            ))),
            CalendarProp::CurrentUserPrivilegeSet => Ok(
                CalendarPropResponse::CurrentUserPrivilegeSet(UserPrivilegeSet::default()),
            ),
        }
    }

    fn get_path(&self) -> &str {
        &self.path
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for CalendarResource<C> {
    type MemberType = CalendarFile;
    type PathComponents = (String, String); // principal, calendar_id
    type File = CalendarFile;

    async fn get_file(&self) -> Result<Self::File> {
        let calendar = self
            .cal_store
            .read()
            .await
            .get_calendar(&self.calendar_id)
            .await
            .map_err(|_e| Error::NotFound)?;
        Ok(CalendarFile {
            calendar,
            principal: self.principal.to_owned(),
            path: self.path.to_owned(),
        })
    }

    async fn get_members(&self, _auth_info: AuthInfo) -> Result<Vec<Self::MemberType>> {
        // As of now the calendar resource has no members since events are shown with REPORT
        Ok(vec![])
    }

    async fn new(
        req: HttpRequest,
        auth_info: AuthInfo,
        path_components: Self::PathComponents,
    ) -> Result<Self, rustical_dav::error::Error> {
        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();

        Ok(Self {
            path: req.path().to_owned(),
            principal: auth_info.user_id,
            calendar_id: path_components.1,
            cal_store,
        })
    }
}
