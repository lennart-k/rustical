use super::methods::{get_event, put_event};
use crate::{CalDavPrincipalUri, Error};
use actix_web::web;
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::{
    extensions::{CommonPropertiesExtension, CommonPropertiesProp},
    privileges::UserPrivilegeSet,
    resource::{PrincipalUri, Resource, ResourceService},
    xml::Resourcetype,
};
use rustical_ical::{CalendarObject, UtcDateTime};
use rustical_store::{CalendarStore, auth::User};
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};
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

#[derive(Clone, From, Into)]
pub struct CalendarObjectResource {
    pub object: CalendarObject,
    pub principal: String,
}

impl Resource for CalendarObjectResource {
    type Prop = CalendarObjectPropWrapper;
    type Error = Error;
    type Principal = User;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &User,
        prop: &CalendarObjectPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CalendarObjectPropWrapperName::CalendarObject(prop) => {
                CalendarObjectPropWrapper::CalendarObject(match prop {
                    CalendarObjectPropName::Getetag => {
                        CalendarObjectProp::Getetag(self.object.get_etag())
                    }
                    CalendarObjectPropName::CalendarData(CalendarData { expand, .. }) => {
                        CalendarObjectProp::CalendarData(if let Some(expand) = expand.as_ref() {
                            self.object.expand_recurrence(
                                Some(expand.start.to_utc()),
                                Some(expand.end.to_utc()),
                            )?
                        } else {
                            self.object.get_ics().to_owned()
                        })
                    }
                    CalendarObjectPropName::Getcontenttype => {
                        CalendarObjectProp::Getcontenttype("text/calendar;charset=utf-8")
                    }
                })
            }
            CalendarObjectPropWrapperName::Common(prop) => CalendarObjectPropWrapper::Common(
                CommonPropertiesExtension::get_prop(self, puri, user, prop)?,
            ),
        })
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_etag(&self) -> Option<String> {
        Some(self.object.get_etag())
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.principal),
        ))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CalendarObjectPathComponents {
    pub principal: String,
    pub calendar_id: String,
    pub object_id: String,
}

#[async_trait]
impl<C: CalendarStore> ResourceService for CalendarObjectResourceService<C> {
    type PathComponents = CalendarObjectPathComponents;
    type Resource = CalendarObjectResource;
    type MemberType = CalendarObjectResource;
    type Error = Error;
    type Principal = User;
    type PrincipalUri = CalDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, calendar-access";

    async fn get_resource(
        &self,
        CalendarObjectPathComponents {
            principal,
            calendar_id,
            object_id,
        }: &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        let object = self
            .cal_store
            .get_object(principal, calendar_id, object_id)
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
            calendar_id,
            object_id,
        }: &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        self.cal_store
            .delete_object(principal, calendar_id, object_id, use_trashbin)
            .await?;
        Ok(())
    }

    fn actix_scope(self) -> actix_web::Scope {
        web::scope("/{object_id}.ics").service(
            self.actix_resource()
                .get(get_event::<C>)
                .put(put_event::<C>),
        )
    }
}
