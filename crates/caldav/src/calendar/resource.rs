use super::methods::mkcalendar::route_mkcalendar;
use super::methods::post::route_post;
use super::methods::report::route_report_calendar;
use super::prop::{
    SupportedCalendarComponentSet, SupportedCalendarData, SupportedReportSet, Transports,
};
use crate::calendar_object::resource::CalendarObjectResource;
use crate::principal::PrincipalResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::http::Method;
use actix_web::web;
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_store::auth::User;
use rustical_store::{Calendar, CalendarStore, SubscriptionStore};
use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::Arc;
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantNames};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, EnumDiscriminants, Clone)]
#[strum_discriminants(
    name(CalendarPropName),
    derive(EnumString, VariantNames, IntoStaticStr),
    strum(serialize_all = "kebab-case")
)]
pub enum CalendarProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", skip_deserializing)]
    Getcontenttype(&'static str),

    // WebDav Push
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    Transports(Transports),
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    Topic(String),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_ICAL")]
    CalendarColor(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarDescription(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarTimezone(Option<String>),
    // https://datatracker.ietf.org/doc/html/rfc7809
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

    // Collection Synchronization (RFC 6578)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SyncToken(String),

    // CalendarServer
    #[xml(ns = "rustical_dav::namespace::NS_CALENDARSERVER")]
    Getctag(String),
    #[xml(ns = "rustical_dav::namespace::NS_CALENDARSERVER")]
    Source(Option<HrefElement>),
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

impl Resource for CalendarResource {
    type PropName = CalendarPropName;
    type Prop = CalendarProp;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> Resourcetype {
        if self.cal.subscription_url.is_none() {
            Resourcetype(&[
                ResourcetypeInner(rustical_dav::namespace::NS_DAV, "collection"),
                ResourcetypeInner(rustical_dav::namespace::NS_CALDAV, "calendar"),
            ])
        } else {
            Resourcetype(&[
                ResourcetypeInner(rustical_dav::namespace::NS_DAV, "collection"),
                ResourcetypeInner(rustical_dav::namespace::NS_CALENDARSERVER, "subscribed"),
            ])
        }
    }

    fn get_prop(
        &self,
        _rmap: &ResourceMap,
        _user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CalendarPropName::Displayname => {
                CalendarProp::Displayname(self.cal.displayname.clone())
            }
            CalendarPropName::CalendarColor => CalendarProp::CalendarColor(self.cal.color.clone()),
            CalendarPropName::CalendarDescription => {
                CalendarProp::CalendarDescription(self.cal.description.clone())
            }
            CalendarPropName::CalendarTimezone => {
                CalendarProp::CalendarTimezone(self.cal.timezone.clone())
            }
            CalendarPropName::CalendarTimezoneId => {
                CalendarProp::CalendarTimezoneId(self.cal.timezone_id.clone())
            }
            CalendarPropName::CalendarOrder => CalendarProp::CalendarOrder(Some(self.cal.order)),
            CalendarPropName::SupportedCalendarComponentSet => {
                CalendarProp::SupportedCalendarComponentSet(SupportedCalendarComponentSet::default())
            }
            CalendarPropName::SupportedCalendarData => {
                CalendarProp::SupportedCalendarData(SupportedCalendarData::default())
            }
            CalendarPropName::Getcontenttype => {
                CalendarProp::Getcontenttype("text/calendar;charset=utf-8")
            }
            CalendarPropName::Transports => CalendarProp::Transports(Default::default()),
            CalendarPropName::Topic => CalendarProp::Topic(self.cal.push_topic.to_owned()),
            CalendarPropName::MaxResourceSize => CalendarProp::MaxResourceSize(10000000),
            CalendarPropName::SupportedReportSet => {
                CalendarProp::SupportedReportSet(SupportedReportSet::default())
            }
            CalendarPropName::SyncToken => CalendarProp::SyncToken(self.cal.format_synctoken()),
            CalendarPropName::Getctag => CalendarProp::Getctag(self.cal.format_synctoken()),
            CalendarPropName::Source => {
                CalendarProp::Source(self.cal.subscription_url.to_owned().map(HrefElement::from))
            }
        })
    }

    fn set_prop(&mut self, prop: Self::Prop) -> Result<(), rustical_dav::Error> {
        if self.read_only {
            return Err(rustical_dav::Error::PropReadOnly);
        }
        match prop {
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
                self.cal.timezone = timezone;
                Ok(())
            }
            CalendarProp::CalendarTimezoneId(timezone_id) => {
                // TODO: Set or remove timezone accordingly
                self.cal.timezone_id = timezone_id;
                Ok(())
            }
            CalendarProp::CalendarOrder(order) => {
                self.cal.order = order.unwrap_or_default();
                Ok(())
            }
            CalendarProp::SupportedCalendarComponentSet(_) => {
                Err(rustical_dav::Error::PropReadOnly)
            }
            CalendarProp::SupportedCalendarData(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Getcontenttype(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Transports(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Topic(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::MaxResourceSize(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::SupportedReportSet(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::SyncToken(_) => Err(rustical_dav::Error::PropReadOnly),
            CalendarProp::Getctag(_) => Err(rustical_dav::Error::PropReadOnly),
            // Converting between a calendar subscription calendar and a normal one would be weird
            CalendarProp::Source(_) => Err(rustical_dav::Error::PropReadOnly),
        }
    }

    fn remove_prop(&mut self, prop: &Self::PropName) -> Result<(), rustical_dav::Error> {
        if self.read_only {
            return Err(rustical_dav::Error::PropReadOnly);
        }
        match prop {
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
            CalendarPropName::Getcontenttype => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::Transports => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::Topic => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::MaxResourceSize => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::SupportedReportSet => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::SyncToken => Err(rustical_dav::Error::PropReadOnly),
            CalendarPropName::Getctag => Err(rustical_dav::Error::PropReadOnly),
            // Converting a calendar subscription calendar into a normal one would be weird
            CalendarPropName::Source => Err(rustical_dav::Error::PropReadOnly),
        }
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.cal.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        if self.cal.subscription_url.is_some() || self.read_only {
            return Ok(UserPrivilegeSet::read_only());
        }

        Ok(UserPrivilegeSet::owner_only(self.cal.principal == user.id))
    }
}

pub struct CalendarResourceService<C: CalendarStore + ?Sized, S: SubscriptionStore + ?Sized> {
    cal_store: Arc<C>,
    __phantom_sub: PhantomData<S>,
}

impl<C: CalendarStore + ?Sized, S: SubscriptionStore + ?Sized> CalendarResourceService<C, S> {
    pub fn new(cal_store: Arc<C>) -> Self {
        Self {
            cal_store,
            __phantom_sub: PhantomData,
        }
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized, S: SubscriptionStore + ?Sized> ResourceService
    for CalendarResourceService<C, S>
{
    type MemberType = CalendarObjectResource;
    type PathComponents = (String, String); // principal, calendar_id
    type Resource = CalendarResource;
    type Error = Error;

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
                    object.get_id().to_string(),
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

    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        let report_method = web::method(Method::from_str("REPORT").unwrap());
        let mkcalendar_method = web::method(Method::from_str("MKCALENDAR").unwrap());

        res.route(report_method.to(route_report_calendar::<C>))
            .route(mkcalendar_method.to(route_mkcalendar::<C>))
            .post(route_post::<C, S>)
    }
}
