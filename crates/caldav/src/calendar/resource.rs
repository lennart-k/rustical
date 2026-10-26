use super::prop::{SupportedCalendarComponentSet, SupportedCalendarData};
use crate::Error;
use crate::calendar::prop::{ReportMethod, SupportedCollationSet};
use caldata::IcalParser;
use caldata::types::CalDateTime;
use chrono::{DateTime, Utc};
use http::Uri;
use rustical_dav::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName, SyncTokenExtension,
    SyncTokenExtensionProp,
};
use rustical_dav::namespace::{NS_CALDAV, NS_CALENDARSERVER, NS_DAV};
use rustical_dav::privileges::{UserPrivilege, UserPrivilegeSet};
use rustical_dav::resource::{PrincipalUri, Resource, ResourceName};
use rustical_dav::resourcetype;
use rustical_dav::xml::{HrefElement, Resourcetype, SupportedReportSet};
use rustical_dav_push::{DavPushExtension, DavPushExtensionProp, VapidPublicKeyB64};
use rustical_store::Access;
use rustical_store::access::{self, Resolved, ResolvedKind};
use rustical_store::auth::Principal;
use rustical_store::{Calendar, CalendarStore};
use rustical_xml::{EnumVariants, PropName};
use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::borrow::Cow;
use std::str::FromStr;

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "CalendarPropName")]
pub enum CalendarProp {
    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_ICAL")]
    CalendarColor(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarDescription(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarTimezone(Option<String>),
    // https://datatracker.ietf.org/doc/html/rfc7809
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", skip_deserializing)]
    TimezoneServiceSet(HrefElement),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarTimezoneId(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_ICAL")]
    CalendarOrder(Option<i64>),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    SupportedCalendarComponentSet(SupportedCalendarComponentSet),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", skip_deserializing)]
    SupportedCalendarData(SupportedCalendarData),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", skip_deserializing)]
    SupportedCollationSet(SupportedCollationSet),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    MaxResourceSize(i64),
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SupportedReportSet(SupportedReportSet<ReportMethod>),
    #[xml(ns = "rustical_dav::namespace::NS_CALENDARSERVER")]
    Source(Option<HrefElement>),
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    MinDateTime(String),
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    MaxDateTime(String),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "CalendarPropWrapperName", untagged)]
pub enum CalendarPropWrapper {
    Calendar(CalendarProp),
    SyncToken(SyncTokenExtensionProp),
    DavPush(DavPushExtensionProp),
    Common(CommonPropertiesProp),
}

#[derive(Clone, Debug)]
pub struct CalendarResource {
    pub cal: Calendar,
    pub read_only: bool,
    pub vapid_pubkey: VapidPublicKeyB64,
    /// The home of the path
    pub home: String,
    /// The name of the calendar in the home (for a share not the owner's id)
    pub name: String,
    /// What the calendar is: the owner, the id and whether the home owns it or it is shared
    pub resolved: Resolved,
}

impl CalendarResource {
    /// The access a sharee has to the calendar itself
    fn copy_privileges(access: Access) -> UserPrivilegeSet {
        match access {
            Access::Read => UserPrivilegeSet::from([
                UserPrivilege::Read,
                UserPrivilege::ReadCurrentUserPrivilegeSet,
                // The sharee's own name, color and order
                UserPrivilege::WriteProperties,
            ]),
            Access::Write => UserPrivilegeSet::from([
                UserPrivilege::Read,
                UserPrivilege::ReadCurrentUserPrivilegeSet,
                UserPrivilege::WriteProperties,
                UserPrivilege::Write,
                UserPrivilege::WriteContent,
            ]),
        }
    }

    /// Calendar properties a sharee may change: only its own view of the share
    const fn is_view_prop(prop: &CalendarPropName) -> bool {
        matches!(
            prop,
            CalendarPropName::CalendarColor | CalendarPropName::CalendarOrder
        )
    }
}

/// Whether the objects of `calendar` are read-only (a subscription or a read-only store)
pub fn objects_read_only<C: CalendarStore + ?Sized>(store: &C, calendar: &Calendar) -> bool {
    calendar.subscription_url.is_some() || store.is_read_only(&calendar.id)
}

/// Test data describes own calendars by the calendar alone
#[cfg(test)]
impl<'de> serde::Deserialize<'de> for CalendarResource {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Data {
            cal: Calendar,
            read_only: bool,
            vapid_pubkey: VapidPublicKeyB64,
        }
        let Data {
            cal,
            read_only,
            vapid_pubkey,
        } = Data::deserialize(deserializer)?;
        Ok(Self {
            home: cal.principal.clone(),
            name: cal.id.clone(),
            resolved: Resolved::for_tests(cal.principal.clone(), cal.id.clone(), ResolvedKind::Own),
            cal,
            read_only,
            vapid_pubkey,
        })
    }
}

impl ResourceName for CalendarResource {
    fn get_name(&self) -> Cow<'_, str> {
        Cow::from(&self.name)
    }
}

impl From<CalendarResource> for Calendar {
    fn from(value: CalendarResource) -> Self {
        value.cal
    }
}

impl SyncTokenExtension for CalendarResource {
    fn get_synctoken(&self) -> String {
        self.cal.format_synctoken()
    }
}

impl DavPushExtension for CalendarResource {
    fn get_topic(&self) -> String {
        self.cal.push_topic.clone()
    }

    fn get_vapid_public_key(&self) -> Option<VapidPublicKeyB64> {
        Some(self.vapid_pubkey.clone())
    }
}

impl Resource for CalendarResource {
    type Prop = CalendarPropWrapper;
    type Error = Error;
    type Principal = Principal;

    fn is_collection(&self) -> bool {
        true
    }

    fn get_resourcetype(&self) -> Resourcetype {
        if self.cal.subscription_url.is_none() {
            resourcetype!((NS_DAV, "collection"), (NS_CALDAV, "calendar"),)
        } else {
            resourcetype!((NS_DAV, "collection"), (NS_CALENDARSERVER, "subscribed"),)
        }
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &Principal,
        prop: &CalendarPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CalendarPropWrapperName::Calendar(prop) => CalendarPropWrapper::Calendar(match prop {
                CalendarPropName::CalendarColor => {
                    CalendarProp::CalendarColor(self.cal.meta.color.clone())
                }
                CalendarPropName::CalendarDescription => {
                    CalendarProp::CalendarDescription(self.cal.meta.description.clone())
                }
                CalendarPropName::CalendarTimezone => {
                    CalendarProp::CalendarTimezone(self.cal.timezone_id.as_ref().and_then(|tzid| {
                        vtimezones_rs::VTIMEZONES
                            .get(tzid)
                            .map(|tz| (*tz).to_string())
                    }))
                }
                // chrono_tz uses the IANA database
                CalendarPropName::TimezoneServiceSet => CalendarProp::TimezoneServiceSet(
                    Uri::from_static("https://www.iana.org/time-zones").into(),
                ),
                CalendarPropName::CalendarTimezoneId => {
                    CalendarProp::CalendarTimezoneId(self.cal.timezone_id.clone())
                }
                CalendarPropName::CalendarOrder => {
                    CalendarProp::CalendarOrder(Some(self.cal.meta.order))
                }
                CalendarPropName::SupportedCalendarComponentSet => {
                    CalendarProp::SupportedCalendarComponentSet(self.cal.components.clone().into())
                }
                CalendarPropName::SupportedCalendarData => {
                    CalendarProp::SupportedCalendarData(SupportedCalendarData::default())
                }
                CalendarPropName::SupportedCollationSet => {
                    CalendarProp::SupportedCollationSet(SupportedCollationSet::default())
                }
                CalendarPropName::MaxResourceSize => CalendarProp::MaxResourceSize(10_000_000),
                CalendarPropName::SupportedReportSet => {
                    CalendarProp::SupportedReportSet(SupportedReportSet::all())
                }
                CalendarPropName::Source => CalendarProp::Source(
                    self.cal
                        .subscription_url
                        .as_deref()
                        .map(Uri::from_str)
                        .and_then(|res| {
                            res.map_or_else(
                                |_| {
                                    tracing::error!(
                                        "Invalid source url for calendar {}/{}",
                                        self.cal.principal,
                                        self.cal.id
                                    );
                                    None
                                },
                                Some,
                            )
                        })
                        .map(HrefElement::from),
                ),
                CalendarPropName::MinDateTime => {
                    CalendarProp::MinDateTime(CalDateTime::from(DateTime::<Utc>::MIN_UTC).format())
                }
                CalendarPropName::MaxDateTime => {
                    CalendarProp::MaxDateTime(CalDateTime::from(DateTime::<Utc>::MAX_UTC).format())
                }
            }),
            CalendarPropWrapperName::SyncToken(prop) => {
                CalendarPropWrapper::SyncToken(SyncTokenExtension::get_prop(self, prop)?)
            }
            CalendarPropWrapperName::DavPush(prop) => {
                CalendarPropWrapper::DavPush(DavPushExtension::get_prop(self, prop)?)
            }
            CalendarPropWrapperName::Common(prop) => CalendarPropWrapper::Common(
                CommonPropertiesExtension::get_prop(self, puri, user, prop)?,
            ),
        })
    }

    fn set_prop(&mut self, prop: Self::Prop) -> Result<(), rustical_dav::Error> {
        if !self.resolved.is_own() {
            let allowed = match &prop {
                CalendarPropWrapper::Calendar(prop) => {
                    Self::is_view_prop(&CalendarPropName::from(prop.clone()))
                }
                CalendarPropWrapper::Common(CommonPropertiesProp::Displayname(_)) => true,
                _ => false,
            };
            if !allowed {
                return Err(rustical_dav::Error::PropReadOnly);
            }
        }
        match prop {
            CalendarPropWrapper::Calendar(prop) => match prop {
                CalendarProp::CalendarColor(color) => {
                    self.cal.meta.color = color;
                    Ok(())
                }
                CalendarProp::CalendarDescription(description) => {
                    self.cal.meta.description = description;
                    Ok(())
                }
                CalendarProp::CalendarTimezone(timezone) => {
                    if let Some(tz) = timezone {
                        // TODO: Proper error (calendar-timezone precondition)
                        let calendar = IcalParser::from_slice(tz.as_bytes())
                            .next()
                            .ok_or_else(|| {
                                rustical_dav::Error::BadRequest(
                                    "No timezone data provided".to_owned(),
                                )
                            })?
                            .map_err(|_| {
                                rustical_dav::Error::BadRequest(
                                    "No timezone data provided".to_owned(),
                                )
                            })?;

                        let timezone = calendar.vtimezones.values().next().ok_or_else(|| {
                            rustical_dav::Error::BadRequest("No timezone data provided".to_owned())
                        })?;
                        let timezone: Option<chrono_tz::Tz> = timezone.into();
                        let timezone = timezone.ok_or_else(|| {
                            rustical_dav::Error::BadRequest("No timezone data provided".to_owned())
                        })?;
                        self.cal.timezone_id = Some(timezone.name().to_owned());
                    }
                    Ok(())
                }
                CalendarProp::CalendarTimezoneId(timezone_id) => {
                    if let Some(tzid) = &timezone_id
                        && !vtimezones_rs::VTIMEZONES.contains_key(tzid)
                    {
                        return Err(rustical_dav::Error::BadRequest(format!(
                            "Invalid timezone-id: {tzid}"
                        )));
                    }
                    self.cal.timezone_id = timezone_id;
                    Ok(())
                }
                CalendarProp::CalendarOrder(order) => {
                    self.cal.meta.order = order.unwrap_or_default();
                    Ok(())
                }
                CalendarProp::SupportedCalendarComponentSet(comp_set) => {
                    self.cal.components = comp_set.into();
                    Ok(())
                }
                CalendarProp::TimezoneServiceSet(_)
                | CalendarProp::SupportedCalendarData(_)
                | CalendarProp::SupportedCollationSet(_)
                | CalendarProp::MaxResourceSize(_)
                | CalendarProp::SupportedReportSet(_)
                | CalendarProp::Source(_)
                | CalendarProp::MinDateTime(_)
                | CalendarProp::MaxDateTime(_) => Err(rustical_dav::Error::PropReadOnly),
            },
            CalendarPropWrapper::SyncToken(prop) => SyncTokenExtension::set_prop(self, prop),
            CalendarPropWrapper::DavPush(prop) => DavPushExtension::set_prop(self, prop),
            CalendarPropWrapper::Common(prop) => CommonPropertiesExtension::set_prop(self, prop),
        }
    }

    fn remove_prop(&mut self, prop: &CalendarPropWrapperName) -> Result<(), rustical_dav::Error> {
        if !self.resolved.is_own() {
            let allowed = match prop {
                CalendarPropWrapperName::Calendar(prop) => Self::is_view_prop(prop),
                CalendarPropWrapperName::Common(CommonPropertiesPropName::Displayname) => true,
                _ => false,
            };
            if !allowed {
                return Err(rustical_dav::Error::PropReadOnly);
            }
        }
        match prop {
            CalendarPropWrapperName::Calendar(prop) => match prop {
                CalendarPropName::CalendarColor => {
                    self.cal.meta.color = None;
                    Ok(())
                }
                CalendarPropName::CalendarDescription => {
                    self.cal.meta.description = None;
                    Ok(())
                }
                CalendarPropName::CalendarTimezone | CalendarPropName::CalendarTimezoneId => {
                    self.cal.timezone_id = None;
                    Ok(())
                }
                CalendarPropName::CalendarOrder => {
                    self.cal.meta.order = 0;
                    Ok(())
                }
                CalendarPropName::SupportedCalendarComponentSet => {
                    Err(rustical_dav::Error::PropReadOnly)
                }
                CalendarPropName::TimezoneServiceSet
                | CalendarPropName::SupportedCalendarData
                | CalendarPropName::SupportedCollationSet
                | CalendarPropName::MaxResourceSize
                | CalendarPropName::SupportedReportSet
                | CalendarPropName::Source
                | CalendarPropName::MinDateTime
                | CalendarPropName::MaxDateTime => Err(rustical_dav::Error::PropReadOnly),
            },
            CalendarPropWrapperName::SyncToken(prop) => SyncTokenExtension::remove_prop(self, prop),
            CalendarPropWrapperName::DavPush(prop) => DavPushExtension::remove_prop(self, prop),
            CalendarPropWrapperName::Common(prop) => {
                CommonPropertiesExtension::remove_prop(self, prop)
            }
        }
    }

    fn get_displayname(&self) -> Option<&str> {
        self.cal.meta.displayname.as_deref()
    }
    fn set_displayname(&mut self, name: Option<String>) -> Result<(), rustical_dav::Error> {
        self.cal.meta.displayname = name;
        Ok(())
    }

    fn get_owner(&self) -> Option<&str> {
        Some(self.resolved.owner())
    }

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        let may_use = access::may_use_home(user, &self.home);
        match self.resolved.kind() {
            ResolvedKind::Copy { access, .. } => Ok(if may_use {
                Self::copy_privileges(*access)
            } else {
                UserPrivilegeSet::default()
            }),
            ResolvedKind::Own => {
                if self.cal.subscription_url.is_some() || self.read_only {
                    return Ok(UserPrivilegeSet::owner_write_properties(may_use));
                }
                Ok(UserPrivilegeSet::owner_only(may_use))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tzdb_version() {
        // Ensure that both chrono_tz and vzic_rs use the same tzdb version
        assert_eq!(
            chrono_tz::IANA_TZDB_VERSION,
            vtimezones_rs::IANA_TZDB_VERSION
        );
    }
}
