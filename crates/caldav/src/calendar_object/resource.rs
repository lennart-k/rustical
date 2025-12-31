use std::borrow::Cow;

use super::prop::{
    CalendarData, CalendarObjectProp, CalendarObjectPropName, CalendarObjectPropWrapper,
    CalendarObjectPropWrapperName,
};
use crate::Error;
use derive_more::derive::{From, Into};
use rustical_dav::{
    extensions::CommonPropertiesExtension,
    privileges::UserPrivilegeSet,
    resource::{PrincipalUri, Resource, ResourceName},
    xml::Resourcetype,
};
use rustical_ical::CalendarObject;
use rustical_store::auth::Principal;

#[derive(Clone, From, Into)]
pub struct CalendarObjectResource {
    pub object: CalendarObject,
    pub principal: String,
}

impl ResourceName for CalendarObjectResource {
    fn get_name(&self) -> Cow<'_, str> {
        Cow::from(format!("{}.ics", self.object.get_id()))
    }
}

impl Resource for CalendarObjectResource {
    type Prop = CalendarObjectPropWrapper;
    type Error = Error;
    type Principal = Principal;

    fn is_collection(&self) -> bool {
        false
    }

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &Principal,
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

    fn get_displayname(&self) -> Option<&str> {
        None
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_etag(&self) -> Option<String> {
        Some(self.object.get_etag())
    }

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.principal),
        ))
    }
}
