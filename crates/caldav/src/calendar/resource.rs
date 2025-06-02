use super::methods::mkcalendar::route_mkcalendar;
use super::methods::post::route_post;
use super::methods::report::route_report_calendar;
use super::prop::{SupportedCalendarComponentSet, SupportedCalendarData, SupportedReportSet};
use crate::calendar_object::resource::{CalendarObjectResource, CalendarObjectResourceService};
use crate::{CalDavPrincipalUri, Error};
use actix_web::http::Method;
use actix_web::web::{self, Data};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_more::derive::{From, Into};
use rustical_dav::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, SyncTokenExtension, SyncTokenExtensionProp,
};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{PrincipalUri, Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_dav_push::{DavPushExtension, DavPushExtensionProp};
use rustical_ical::CalDateTime;
use rustical_store::auth::User;
use rustical_store::{Calendar, CalendarStore, SubscriptionStore};
use rustical_xml::{EnumUnitVariants, EnumVariants};
use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "CalendarPropName")]
pub enum CalendarProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(Option<String>),

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
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", skip_deserializing)]
    SupportedCalendarComponentSet(SupportedCalendarComponentSet),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV", skip_deserializing)]
    SupportedCalendarData(SupportedCalendarData),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    MaxResourceSize(i64),
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    SupportedReportSet(SupportedReportSet),
    #[xml(ns = "rustical_dav::namespace::NS_CALENDARSERVER")]
    Source(Option<HrefElement>),
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    MinDateTime(String),
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    MaxDateTime(String),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "CalendarPropWrapperName", untagged)]
pub enum CalendarPropWrapper {
    Calendar(CalendarProp),
    SyncToken(SyncTokenExtensionProp),
    DavPush(DavPushExtensionProp),
    Common(CommonPropertiesProp),
}

#[derive(Clone, Debug, From, Into)]
pub struct CalendarResource {
    pub cal: Calendar,
    pub read_only: bool,
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
        self.cal.push_topic.to_owned()
    }
}

impl Resource for CalendarResource {
    type Prop = CalendarPropWrapper;
    type Error = Error;
    type Principal = User;

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
        user: &User,
        prop: &CalendarPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CalendarPropWrapperName::Calendar(prop) => CalendarPropWrapper::Calendar(match prop {
                CalendarPropName::Displayname => {
                    CalendarProp::Displayname(self.cal.displayname.clone())
                }
                CalendarPropName::CalendarColor => {
                    CalendarProp::CalendarColor(self.cal.color.clone())
                }
                CalendarPropName::CalendarDescription => {
                    CalendarProp::CalendarDescription(self.cal.description.clone())
                }
                CalendarPropName::CalendarTimezone => {
                    CalendarProp::CalendarTimezone(self.cal.timezone.clone())
                }
                // chrono_tz uses the IANA database
                CalendarPropName::TimezoneServiceSet => CalendarProp::TimezoneServiceSet(
                    "https://www.iana.org/time-zones".to_owned().into(),
                ),
                CalendarPropName::CalendarTimezoneId => {
                    CalendarProp::CalendarTimezoneId(self.cal.timezone_id.clone())
                }
                CalendarPropName::CalendarOrder => {
                    CalendarProp::CalendarOrder(Some(self.cal.order))
                }
                CalendarPropName::SupportedCalendarComponentSet => {
                    CalendarProp::SupportedCalendarComponentSet(self.cal.components.clone().into())
                }
                CalendarPropName::SupportedCalendarData => {
                    CalendarProp::SupportedCalendarData(SupportedCalendarData::default())
                }
                CalendarPropName::MaxResourceSize => CalendarProp::MaxResourceSize(10000000),
                CalendarPropName::SupportedReportSet => {
                    CalendarProp::SupportedReportSet(SupportedReportSet::default())
                }
                CalendarPropName::Source => CalendarProp::Source(
                    self.cal.subscription_url.to_owned().map(HrefElement::from),
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
        if self.read_only {
            return Err(rustical_dav::Error::PropReadOnly);
        }
        match prop {
            CalendarPropWrapper::Calendar(prop) => match prop {
                CalendarProp::Displayname(displayname) => {
                    self.cal.displayname = displayname;
                    Ok(())
                }
                CalendarProp::CalendarColor(color) => {
                    self.cal.color = color;
                    Ok(())
                }
                CalendarProp::CalendarDescription(description) => {
                    self.cal.description = description;
                    Ok(())
                }
                CalendarProp::CalendarTimezone(timezone) => {
                    // TODO: Ensure that timezone-id is also updated
                    self.cal.timezone = timezone;
                    Ok(())
                }
                CalendarProp::TimezoneServiceSet(_) => Err(rustical_dav::Error::PropReadOnly),
                CalendarProp::CalendarTimezoneId(timezone_id) => {
                    if let Some(tzid) = &timezone_id {
                        // Validate timezone id
                        chrono_tz::Tz::from_str(tzid).map_err(|_| {
                            rustical_dav::Error::BadRequest(format!(
                                "Invalid timezone-id: {}",
                                tzid
                            ))
                        })?;
                        // TODO: Ensure that timezone is also updated (For now hope that clients play nice)
                    }
                    self.cal.timezone_id = timezone_id;
                    Ok(())
                }
                CalendarProp::CalendarOrder(order) => {
                    self.cal.order = order.unwrap_or_default();
                    Ok(())
                }
                CalendarProp::SupportedCalendarComponentSet(comp_set) => {
                    self.cal.components = comp_set.into();
                    Ok(())
                }
                CalendarProp::SupportedCalendarData(_) => Err(rustical_dav::Error::PropReadOnly),
                CalendarProp::MaxResourceSize(_) => Err(rustical_dav::Error::PropReadOnly),
                CalendarProp::SupportedReportSet(_) => Err(rustical_dav::Error::PropReadOnly),
                // Converting between a calendar subscription calendar and a normal one would be weird
                CalendarProp::Source(_) => Err(rustical_dav::Error::PropReadOnly),
                CalendarProp::MinDateTime(_) => Err(rustical_dav::Error::PropReadOnly),
                CalendarProp::MaxDateTime(_) => Err(rustical_dav::Error::PropReadOnly),
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
                CalendarPropName::Displayname => {
                    self.cal.displayname = None;
                    Ok(())
                }
                CalendarPropName::CalendarColor => {
                    self.cal.color = None;
                    Ok(())
                }
                CalendarPropName::CalendarDescription => {
                    self.cal.description = None;
                    Ok(())
                }
                CalendarPropName::CalendarTimezone => {
                    self.cal.timezone = None;
                    Ok(())
                }
                CalendarPropName::TimezoneServiceSet => Err(rustical_dav::Error::PropReadOnly),
                CalendarPropName::CalendarTimezoneId => {
                    self.cal.timezone_id = None;
                    Ok(())
                }
                CalendarPropName::CalendarOrder => {
                    self.cal.order = 0;
                    Ok(())
                }
                CalendarPropName::SupportedCalendarComponentSet => {
                    Err(rustical_dav::Error::PropReadOnly)
                }
                CalendarPropName::SupportedCalendarData => Err(rustical_dav::Error::PropReadOnly),
                CalendarPropName::MaxResourceSize => Err(rustical_dav::Error::PropReadOnly),
                CalendarPropName::SupportedReportSet => Err(rustical_dav::Error::PropReadOnly),
                // Converting a calendar subscription calendar into a normal one would be weird
                CalendarPropName::Source => Err(rustical_dav::Error::PropReadOnly),
                CalendarPropName::MinDateTime => Err(rustical_dav::Error::PropReadOnly),
                CalendarPropName::MaxDateTime => Err(rustical_dav::Error::PropReadOnly),
            },
            CalendarPropWrapperName::SyncToken(prop) => SyncTokenExtension::remove_prop(self, prop),
            CalendarPropWrapperName::DavPush(prop) => DavPushExtension::remove_prop(self, prop),
            CalendarPropWrapperName::Common(prop) => {
                CommonPropertiesExtension::remove_prop(self, prop)
            }
        }
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.cal.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        if self.cal.subscription_url.is_some() || self.read_only {
            return Ok(UserPrivilegeSet::owner_read(
                user.is_principal(&self.cal.principal),
            ));
        }

        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.cal.principal),
        ))
    }
}

pub struct CalendarResourceService<C: CalendarStore, S: SubscriptionStore> {
    cal_store: Arc<C>,
    sub_store: Arc<S>,
}

impl<C: CalendarStore, S: SubscriptionStore> CalendarResourceService<C, S> {
    pub fn new(cal_store: Arc<C>, sub_store: Arc<S>) -> Self {
        Self {
            cal_store,
            sub_store,
        }
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore, S: SubscriptionStore> ResourceService for CalendarResourceService<C, S> {
    type MemberType = CalendarObjectResource;
    type PathComponents = (String, String); // principal, calendar_id
    type Resource = CalendarResource;
    type Error = Error;
    type Principal = User;
    type PrincipalUri = CalDavPrincipalUri;

    async fn get_resource(
        &self,
        (principal, cal_id): &Self::PathComponents,
    ) -> Result<Self::Resource, Error> {
        let calendar = self.cal_store.get_calendar(principal, cal_id).await?;
        Ok(CalendarResource {
            cal: calendar,
            read_only: self.cal_store.is_read_only(),
        })
    }

    async fn get_members(
        &self,
        (principal, cal_id): &Self::PathComponents,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(self
            .cal_store
            .get_objects(principal, cal_id)
            .await?
            .into_iter()
            .map(|object| {
                (
                    format!("{}.ics", object.get_id()),
                    CalendarObjectResource {
                        object,
                        principal: principal.to_owned(),
                    },
                )
            })
            .collect())
    }

    async fn save_resource(
        &self,
        (principal, cal_id): &Self::PathComponents,
        file: Self::Resource,
    ) -> Result<(), Self::Error> {
        self.cal_store
            .update_calendar(principal.to_owned(), cal_id.to_owned(), file.into())
            .await?;
        Ok(())
    }

    async fn delete_resource(
        &self,
        (principal, cal_id): &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        self.cal_store
            .delete_calendar(principal, cal_id, use_trashbin)
            .await?;
        Ok(())
    }

    fn actix_scope(self) -> actix_web::Scope {
        let report_method = web::method(Method::from_str("REPORT").unwrap());
        let mkcalendar_method = web::method(Method::from_str("MKCALENDAR").unwrap());
        web::scope("/{calendar_id}")
            .app_data(Data::from(self.sub_store.clone()))
            .service(CalendarObjectResourceService::new(self.cal_store.clone()).actix_scope())
            .service(
                self.actix_resource()
                    .route(report_method.to(route_report_calendar::<C>))
                    .route(mkcalendar_method.to(route_mkcalendar::<C>))
                    .post(route_post::<C, S>),
            )
    }
}
