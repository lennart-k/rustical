use super::methods::{get_event, put_event};
use crate::{principal::PrincipalResource, Error};
use actix_web::{dev::ResourceMap, web::Data, HttpRequest};
use async_trait::async_trait;
use derive_more::derive::{From, Into, TryInto};
use rustical_dav::{
    extension::BoxedExtension,
    extensions::{CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName},
    privileges::UserPrivilegeSet,
    resource::{InvalidProperty, Resource, ResourceService},
    xml::HrefElement,
};
use rustical_store::{auth::User, CalendarObject, CalendarStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, VariantNames};

pub struct CalendarObjectResourceService<C: CalendarStore + ?Sized> {
    pub cal_store: Arc<C>,
    pub path: String,
    pub principal: String,
    pub cal_id: String,
    pub object_id: String,
}

#[derive(EnumString, VariantNames, Clone, From, TryInto)]
#[strum(serialize_all = "kebab-case")]
pub enum CalendarObjectPropName {
    Getetag,
    CalendarData,
    Getcontenttype,
    Owner,
    #[from]
    #[try_into]
    #[strum(disabled)]
    ExtCommonProperties(CommonPropertiesPropName),
}

#[derive(Deserialize, Serialize, From, TryInto)]
#[serde(rename_all = "kebab-case")]
pub enum CalendarObjectProp {
    // WebDAV (RFC 2518)
    Getetag(String),
    Getcontenttype(String),

    // CalDAV (RFC 4791)
    #[serde(rename = "C:calendar-data")]
    CalendarData(String),

    // WebDAV Access Control (RFC 3744)
    Owner(HrefElement),

    #[serde(skip_deserializing, untagged)]
    #[from]
    #[try_into]
    ExtCommonProperties(CommonPropertiesProp<CalendarObjectResource>),

    #[serde(untagged)]
    Invalid,
}

impl InvalidProperty for CalendarObjectProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(Clone, From, Into)]
pub struct CalendarObjectResource {
    pub object: CalendarObject,
    pub principal: String,
}

// TODO: set correct resourcetype
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    collection: (),
}

impl Resource for CalendarObjectResource {
    type PropName = CalendarObjectPropName;
    type Prop = CalendarObjectProp;
    type Error = Error;
    type ResourceType = Resourcetype;

    fn list_extensions() -> Vec<BoxedExtension<Self>> {
        vec![BoxedExtension::from_ext(CommonPropertiesExtension::<
            PrincipalResource,
        >::default())]
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        _user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CalendarObjectPropName::Getetag => CalendarObjectProp::Getetag(self.object.get_etag()),
            CalendarObjectPropName::CalendarData => {
                CalendarObjectProp::CalendarData(self.object.get_ics().to_owned())
            }
            CalendarObjectPropName::Getcontenttype => {
                CalendarObjectProp::Getcontenttype("text/calendar;charset=utf-8".to_owned())
            }
            CalendarObjectPropName::Owner => CalendarObjectProp::Owner(
                PrincipalResource::get_principal_url(rmap, &self.principal).into(),
            ),
            _ => panic!("we shouldn't end up here"),
        })
    }

    #[inline]
    fn resource_name() -> &'static str {
        "caldav_calendar_object"
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(self.principal == user.id))
    }
}

#[derive(Debug, Clone)]
pub struct CalendarObjectPathComponents {
    pub principal: String,
    pub cal_id: String,
    pub object_id: String,
}

impl<'de> Deserialize<'de> for CalendarObjectPathComponents {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        type Inner = (String, String, String);
        let (principal, calendar, mut object) = Inner::deserialize(deserializer)?;
        if object.ends_with(".ics") {
            object.truncate(object.len() - 4);
        }
        Ok(Self {
            principal,
            cal_id: calendar,
            object_id: object,
        })
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for CalendarObjectResourceService<C> {
    type PathComponents = CalendarObjectPathComponents;
    type Resource = CalendarObjectResource;
    type MemberType = CalendarObjectResource;
    type Error = Error;

    async fn new(
        req: &HttpRequest,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let CalendarObjectPathComponents {
            principal,
            cal_id,
            object_id,
        } = path_components;

        let cal_store = req
            .app_data::<Data<C>>()
            .expect("no calendar store in app_data!")
            .clone()
            .into_inner();

        Ok(Self {
            cal_store,
            principal,
            cal_id,
            object_id,
            path: req.path().to_string(),
        })
    }

    async fn get_resource(&self) -> Result<Self::Resource, Self::Error> {
        let object = self
            .cal_store
            .get_object(&self.principal, &self.cal_id, &self.object_id)
            .await?;
        Ok(CalendarObjectResource {
            object,
            principal: self.principal.to_owned(),
        })
    }

    async fn save_resource(&self, _file: Self::Resource) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }

    async fn delete_resource(&self, use_trashbin: bool) -> Result<(), Self::Error> {
        self.cal_store
            .delete_object(&self.principal, &self.cal_id, &self.object_id, use_trashbin)
            .await?;
        Ok(())
    }

    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        res.get(get_event::<C>).put(put_event::<C>)
    }
}
