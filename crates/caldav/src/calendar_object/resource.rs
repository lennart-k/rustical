use super::methods::{get_event, put_event};
use crate::{principal::PrincipalResource, Error};
use actix_web::dev::ResourceMap;
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::{
    extensions::{CommonPropertiesExtension, CommonPropertiesProp},
    privileges::UserPrivilegeSet,
    resource::{Resource, ResourceService},
    xml::Resourcetype,
};
use rustical_store::{auth::User, CalendarObject, CalendarStore};
use rustical_xml::{EnumUnitVariants, EnumVariants, XmlDeserialize, XmlSerialize};
use serde::Deserialize;
use std::sync::Arc;

pub struct CalendarObjectResourceService<C: CalendarStore> {
    cal_store: Arc<C>,
}

impl<C: CalendarStore> CalendarObjectResourceService<C> {
    pub fn new(cal_store: Arc<C>) -> Self {
        Self { cal_store }
    }
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "CalendarObjectPropName")]
pub enum CalendarObjectProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Getetag(String),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", skip_deserializing)]
    Getcontenttype(&'static str),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarData(String),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "CalendarObjectPropWrapperName", untagged)]
pub enum CalendarObjectPropWrapper {
    CalendarObject(CalendarObjectProp),
    Common(CommonPropertiesProp),
}

#[derive(Clone, From, Into)]
pub struct CalendarObjectResource {
    pub object: CalendarObject,
    pub principal: String,
}

impl Resource for CalendarObjectResource {
    type PropName = CalendarObjectPropWrapperName;
    type Prop = CalendarObjectPropWrapper;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[])
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CalendarObjectPropWrapperName::CalendarObject(prop) => {
                CalendarObjectPropWrapper::CalendarObject(match prop {
                    CalendarObjectPropName::Getetag => {
                        CalendarObjectProp::Getetag(self.object.get_etag())
                    }
                    CalendarObjectPropName::CalendarData => {
                        CalendarObjectProp::CalendarData(self.object.get_ics().to_owned())
                    }
                    CalendarObjectPropName::Getcontenttype => {
                        CalendarObjectProp::Getcontenttype("text/calendar;charset=utf-8")
                    }
                })
            }
            CalendarObjectPropWrapperName::Common(prop) => CalendarObjectPropWrapper::Common(
                CommonPropertiesExtension::get_prop(self, rmap, user, prop)?,
            ),
        })
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
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
impl<C: CalendarStore> ResourceService for CalendarObjectResourceService<C> {
    type PathComponents = CalendarObjectPathComponents;
    type Resource = CalendarObjectResource;
    type MemberType = CalendarObjectResource;
    type Error = Error;

    async fn get_resource(
        &self,
        CalendarObjectPathComponents {
            principal,
            cal_id,
            object_id,
        }: &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        let object = self
            .cal_store
            .get_object(principal, cal_id, object_id)
            .await?;
        Ok(CalendarObjectResource {
            object,
            principal: principal.to_owned(),
        })
    }

    async fn delete_resource(
        &self,
        CalendarObjectPathComponents {
            principal,
            cal_id,
            object_id,
        }: &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        self.cal_store
            .delete_object(principal, cal_id, object_id, use_trashbin)
            .await?;
        Ok(())
    }

    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        res.get(get_event::<C>).put(put_event::<C>)
    }
}
