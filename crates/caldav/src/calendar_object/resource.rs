use super::prop::{
    CalendarData, CalendarObjectProp, CalendarObjectPropName, CalendarObjectPropWrapper,
    CalendarObjectPropWrapperName,
};
use crate::Error;
use caldata::generator::Emitter;
use rustical_dav::{
    extensions::CommonPropertiesExtension,
    privileges::{UserPrivilege, UserPrivilegeSet},
    resource::{PrincipalUri, Resource, ResourceName},
    resourcetype,
    xml::Resourcetype,
};
use rustical_ical::CalendarObject;
use rustical_store::Access;
use rustical_store::access::{self, Resolved, ResolvedKind};
use rustical_store::auth::Principal;
use rustical_store::shared::FULL_LEVEL;
use std::borrow::Cow;

#[derive(Clone)]
pub struct CalendarObjectResource {
    /// The object as the path shows it (for a share the sharee's view)
    pub object: CalendarObject,
    pub object_id: String,
    /// The home of the path
    pub principal: String,
    /// The calendar's objects are read-only (subscription or read-only store)
    pub read_only: bool,
    /// The calendar the object belongs to
    pub resolved: Resolved,
    /// The visibility level of the object in `resolved` (always full in an own calendar)
    pub level: u8,
}

impl CalendarObjectResource {
    #[must_use]
    pub fn new(
        object: CalendarObject,
        object_id: String,
        home: &str,
        resolved: &Resolved,
        read_only: bool,
        level: u8,
    ) -> Self {
        Self {
            object,
            object_id,
            principal: home.to_owned(),
            read_only,
            resolved: resolved.clone(),
            level,
        }
    }
}

impl ResourceName for CalendarObjectResource {
    fn get_name(&self) -> Cow<'_, str> {
        Cow::from(format!("{}.ics", self.object_id))
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
        resourcetype!()
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
                    // expanding on a PROPFIND is not defined by RFC 4791.
                    // We still do it but return a NotFound error if there's no matching occurence
                    // In report methods we can catch the error and ignore it
                    CalendarObjectPropName::CalendarData(CalendarData { expand, .. }) => {
                        CalendarObjectProp::CalendarData(expand.as_ref().map_or_else(
                            || Ok::<_, Error>(self.object.get_ics().to_owned()),
                            |expand| {
                                Ok(self
                                    .object
                                    .get_inner()
                                    .expand_recurrence(
                                        Some(expand.start.to_utc()),
                                        Some(expand.end.to_utc()),
                                    )
                                    .ok_or(Error::NotFound)?
                                    .generate_string())
                            },
                        )?)
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
        Some(self.resolved.owner())
    }

    fn get_etag(&self) -> Option<String> {
        Some(self.object.get_etag())
    }

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        let may_use = access::may_use_home(user, &self.principal);
        if self.read_only {
            return Ok(UserPrivilegeSet::owner_read(may_use));
        }
        match self.resolved.kind() {
            ResolvedKind::Own => Ok(UserPrivilegeSet::owner_only(may_use)),
            // A sharee changes only what it sees in full
            ResolvedKind::Copy {
                access: Access::Write,
                ..
            } if may_use && self.level >= FULL_LEVEL => Ok(UserPrivilegeSet::from([
                UserPrivilege::Read,
                UserPrivilege::ReadCurrentUserPrivilegeSet,
                UserPrivilege::Write,
                UserPrivilege::WriteProperties,
                UserPrivilege::WriteContent,
            ])),
            ResolvedKind::Copy { .. } => Ok(UserPrivilegeSet::owner_read(may_use)),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use rustical_dav::resource::Resource;
    use rustical_ical::{CalendarObject, UtcDateTime};
    use rustical_store::auth::Principal;

    use crate::{
        CalDavPrincipalUri, Error,
        calendar_object::{
            CalendarData, CalendarObjectPropName, CalendarObjectPropWrapperName, ExpandElement,
            resource::CalendarObjectResource,
        },
    };

    #[test]
    fn test_expand_out_of_range() {
        let resource = CalendarObjectResource {
            principal: "user@rustical.dev".to_string(),
            object_id: "asd".to_string(),
            read_only: false,
            resolved: rustical_store::access::Resolved::for_tests(
                "user@rustical.dev".to_string(),
                "cal".to_string(),
                rustical_store::access::ResolvedKind::Own,
            ),
            level: rustical_store::shared::FULL_LEVEL,
            object: CalendarObject::from_ics(
                r"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Example Corp.//CalDAV Client//EN
BEGIN:VEVENT
UID:20010712T182145Z-123401@example.com
DTSTAMP:20060712T182145Z
DTSTART:20060714T170000Z
RRULE:FREQ=YEARLY;COUNT=1
DTEND:20060715T040000Z
SUMMARY:Bastille Day Party
END:VEVENT
END:VCALENDAR"
                    .to_string(),
            )
            .unwrap(),
        };
        assert!(matches!(
            resource.get_prop(
                &CalDavPrincipalUri("/"),
                &Principal {
                    id: "user@rustical.dev".to_string(),
                    displayname: None,
                    principal_type: rustical_store::auth::PrincipalType::Individual,
                    password: None,
                    memberships: vec![],
                },
                &CalendarObjectPropWrapperName::CalendarObject(
                    CalendarObjectPropName::CalendarData(CalendarData {
                        expand: Some(ExpandElement {
                            start: UtcDateTime(Utc.with_ymd_and_hms(1900, 1, 1, 0, 0, 0).unwrap()),
                            end: UtcDateTime(Utc.with_ymd_and_hms(1901, 1, 1, 0, 0, 0).unwrap()),
                        }),
                        ..Default::default()
                    },)
                ),
            ),
            Err(Error::NotFound)
        ));
    }
}
