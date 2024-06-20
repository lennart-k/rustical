use crate::Error;
use actix_web::{web::Data, HttpRequest};
use anyhow::anyhow;
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml_snippets::{HrefElement, TextNode};
use rustical_store::calendar::Calendar;
use rustical_store::CalendarStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, IntoStaticStr, VariantNames};
use tokio::sync::RwLock;

pub struct CalendarResource<C: CalendarStore + ?Sized> {
    pub cal_store: Arc<RwLock<C>>,
    pub path: String,
    pub principal: String,
    pub calendar_id: String,
}

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

#[derive(EnumString, Debug, VariantNames, IntoStaticStr, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum CalendarPropName {
    Resourcetype,
    CurrentUserPrincipal,
    Owner,
    Displayname,
    CalendarColor,
    CalendarDescription,
    CalendarOrder,
    SupportedCalendarComponentSet,
    SupportedCalendarData,
    Getcontenttype,
    CurrentUserPrivilegeSet,
    MaxResourceSize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CalendarProp {
    Resourcetype(Resourcetype),
    CurrentUserPrincipal(HrefElement),
    Owner(HrefElement),
    Displayname(TextNode),
    #[serde(rename = "IC:calendar-color", alias = "calendar-color")]
    CalendarColor(TextNode),
    #[serde(rename = "C:calendar-description", alias = "calendar-description")]
    CalendarDescription(TextNode),
    #[serde(rename = "IC:calendar-description", alias = "calendar-description")]
    CalendarOrder(TextNode),
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
    #[serde(other)]
    Invalid,
}

impl InvalidProperty for CalendarProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(Clone, Debug)]
pub struct CalendarFile {
    pub calendar: Calendar,
    pub principal: String,
    pub path: String,
}

impl Resource for CalendarFile {
    type PropName = CalendarPropName;
    type Prop = CalendarProp;
    type Error = Error;

    fn get_prop(&self, prefix: &str, prop: Self::PropName) -> Result<Self::Prop, Self::Error> {
        match prop {
            CalendarPropName::Resourcetype => {
                Ok(CalendarProp::Resourcetype(Resourcetype::default()))
            }
            CalendarPropName::CurrentUserPrincipal => Ok(CalendarProp::CurrentUserPrincipal(
                HrefElement::new(format!("{}/{}/", prefix, self.principal)),
            )),
            CalendarPropName::Owner => Ok(CalendarProp::Owner(HrefElement::new(format!(
                "{}/{}/",
                prefix, self.principal
            )))),
            CalendarPropName::Displayname => Ok(CalendarProp::Displayname(TextNode(
                self.calendar.name.clone(),
            ))),
            CalendarPropName::CalendarColor => Ok(CalendarProp::CalendarColor(TextNode(
                self.calendar.color.clone(),
            ))),
            CalendarPropName::CalendarDescription => Ok(CalendarProp::CalendarDescription(
                TextNode(self.calendar.description.clone()),
            )),
            CalendarPropName::CalendarOrder => Ok(CalendarProp::CalendarOrder(TextNode(
                format!("{}", self.calendar.order).into(),
            ))),
            CalendarPropName::SupportedCalendarComponentSet => Ok(
                CalendarProp::SupportedCalendarComponentSet(SupportedCalendarComponentSet {
                    comp: vec![SupportedCalendarComponent {
                        name: "VEVENT".to_owned(),
                    }],
                }),
            ),
            CalendarPropName::SupportedCalendarData => Ok(CalendarProp::SupportedCalendarData(
                SupportedCalendarData::default(),
            )),
            CalendarPropName::Getcontenttype => Ok(CalendarProp::Getcontenttype(TextNode(Some(
                "text/calendar;charset=utf-8".to_owned(),
            )))),
            CalendarPropName::MaxResourceSize => Ok(CalendarProp::MaxResourceSize(TextNode(Some(
                "10000000".to_owned(),
            )))),
            CalendarPropName::CurrentUserPrivilegeSet => Ok(CalendarProp::CurrentUserPrivilegeSet(
                UserPrivilegeSet::default(),
            )),
        }
    }

    fn set_prop(&mut self, prop: Self::Prop) -> Result<(), rustical_dav::Error> {
        match prop {
            CalendarProp::CalendarColor(color) => {
                self.calendar.color = color.0;
            }
            CalendarProp::Displayname(TextNode(name)) => {
                self.calendar.name = name;
            }
            _ => return Err(rustical_dav::Error::PropReadOnly),
        }
        Ok(())
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
    type Error = Error;

    async fn get_file(&self) -> Result<Self::File, Error> {
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

    async fn get_members(
        &self,
        _auth_info: AuthInfo,
    ) -> Result<Vec<Self::MemberType>, Self::Error> {
        // As of now the calendar resource has no members since events are shown with REPORT
        Ok(vec![])
    }

    async fn new(
        req: HttpRequest,
        auth_info: AuthInfo,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
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

    async fn save_file(&self, file: Self::File) -> Result<(), Self::Error> {
        self.cal_store
            .write()
            .await
            .update_calendar(self.calendar_id.to_owned(), file.calendar)
            .await?;
        Ok(())
    }
}
