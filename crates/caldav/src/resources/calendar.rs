use actix_web::{web::Data, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::error::Error;
use rustical_dav::{
    resource::Resource,
    xml_snippets::{HrefElement, TextNode},
};
use rustical_store::calendar::{Calendar, CalendarStore};
use serde::Serialize;
use std::sync::Arc;
use strum::{EnumString, IntoStaticStr, VariantNames};
use tokio::sync::RwLock;

pub struct CalendarResource<C: CalendarStore + ?Sized> {
    pub cal_store: Arc<RwLock<C>>,
    pub calendar: Calendar,
    pub path: String,
    pub prefix: String,
    pub principal: String,
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

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> Resource for CalendarResource<C> {
    type MemberType = Self;
    type UriComponents = (String, String); // principal, calendar_id
    type PropType = CalendarProp;
    type PropResponse = CalendarPropResponse;

    async fn acquire_from_request(
        req: HttpRequest,
        _auth_info: AuthInfo,
        uri_components: Self::UriComponents,
        prefix: String,
    ) -> Result<Self, Error> {
        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();

        let (principal, cid) = uri_components;
        // TODO: fix errors
        let calendar = cal_store
            .read()
            .await
            .get_calendar(&cid)
            .await
            .map_err(|_e| Error::NotFound)?;
        Ok(Self {
            cal_store,
            calendar,
            path: req.path().to_string(),
            prefix,
            principal,
        })
    }

    fn get_path(&self) -> &str {
        &self.path
    }

    async fn get_members(&self) -> Result<Vec<Self::MemberType>> {
        // As of now the calendar resource has no members
        Ok(vec![])
    }

    fn get_prop(&self, prop: Self::PropType) -> Result<Self::PropResponse> {
        match prop {
            CalendarProp::Resourcetype => {
                Ok(CalendarPropResponse::Resourcetype(Resourcetype::default()))
            }
            CalendarProp::CurrentUserPrincipal => Ok(CalendarPropResponse::CurrentUserPrincipal(
                HrefElement::new(format!("{}/{}/", self.prefix, self.principal)),
            )),
            CalendarProp::Owner => Ok(CalendarPropResponse::Owner(HrefElement::new(format!(
                "{}/{}/",
                self.prefix, self.principal
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
}
