use super::prop::{SupportedCalendarComponentSet, SupportedCalendarData};
use crate::Error;
use crate::calendar::prop::{ReportMethod, SupportedCollationSet};
use chrono::{DateTime, Utc};
use derive_more::derive::{From, Into};
use ical::IcalParser;
use rustical_dav::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, SyncTokenExtension, SyncTokenExtensionProp,
};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{PrincipalUri, Resource, ResourceName};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner, SupportedReportSet};
use rustical_dav_push::{DavPushExtension, DavPushExtensionProp};
use rustical_ical::CalDateTime;
use rustical_store::Calendar;
use rustical_store::auth::Principal;
use rustical_xml::{EnumVariants, PropName};
use rustical_xml::{XmlDeserialize, XmlSerialize};
use serde::Deserialize;

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
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
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

#[derive(Clone, Debug, From, Into, Deserialize)]
pub struct CalendarResource {
    pub cal: Calendar,
    pub read_only: bool,
}

impl ResourceName for CalendarResource {
    fn get_name(&self) -> String {
        self.cal.id.clone()
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
            Resourcetype(&[
                ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "collection"),
                ResourcetypeInner(Some(rustical_dav::namespace::NS_CALDAV), "calendar"),
            ])
        } else {
            Resourcetype(&[
                ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "collection"),
                ResourcetypeInner(
                    Some(rustical_dav::namespace::NS_CALENDARSERVER),
                    "subscribed",
                ),
            ])
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
                    "https://www.iana.org/time-zones".to_owned().into(),
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
                CalendarPropName::Source => {
                    CalendarProp::Source(self.cal.subscription_url.clone().map(HrefElement::from))
                }
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
        if self.read_only {
            return Err(rustical_dav::Error::PropReadOnly);
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
                        let calendar = IcalParser::new(tz.as_bytes())
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

                        let timezone = calendar.timezones.first().ok_or_else(|| {
                            rustical_dav::Error::BadRequest("No timezone data provided".to_owned())
                        })?;
                        let timezone: chrono_tz::Tz = timezone.try_into().map_err(|_| {
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
        if self.read_only {
            return Err(rustical_dav::Error::PropReadOnly);
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
        Some(&self.cal.principal)
    }

    fn get_user_privileges(&self, user: &Principal) -> Result<UserPrivilegeSet, Self::Error> {
        if self.cal.subscription_url.is_some() || self.read_only {
            return Ok(UserPrivilegeSet::owner_write_properties(
                user.is_principal(&self.cal.principal),
            ));
        }

        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.cal.principal),
        ))
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
