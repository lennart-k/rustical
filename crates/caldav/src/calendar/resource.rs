use super::methods::mkcalendar::route_mkcalendar;
use super::methods::post::route_post;
use super::methods::report::route_report_calendar;
use super::prop::{
    SupportedCalendarComponent, SupportedCalendarComponentSet, SupportedCalendarData,
    SupportedReportSet, Transports,
};
use crate::calendar_object::resource::CalendarObjectResource;
use crate::principal::PrincipalResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::http::Method;
use actix_web::web;
use actix_web::{web::Data, HttpRequest};
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::auth::User;
use rustical_store::{Calendar, CalendarStore};
use rustical_xml::{XmlDeserialize, XmlSerialize};
use sha2::{Digest, Sha256};
use std::str::FromStr;
use std::sync::Arc;
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantNames};

pub struct CalendarResourceService<C: CalendarStore + ?Sized> {
    cal_store: Arc<C>,
    principal: String,
    calendar_id: String,
}

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
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Getcontenttype(String),

    // WebDav Push
    // NOTE: Here we implement an older version of the spec since the new property name is not reflected
    // in DAVx5 yet
    // https://github.com/bitfireAT/webdav-push/commit/461259a2f2174454b2b00033419b11fac52b79e3
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    Transports(Transports),
    Topic(String),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_ICAL")]
    CalendarColor(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarDescription(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    CalendarTimezone(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_ICAL")]
    CalendarOrder(Option<i64>),
    // TODO: Re-add
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    #[xml(skip_deserializing)]
    SupportedCalendarComponentSet(SupportedCalendarComponentSet),
    #[xml(ns = "rustical_dav::namespace::NS_CALDAV")]
    #[xml(skip_deserializing)]
    SupportedCalendarData(SupportedCalendarData),
    MaxResourceSize(i64),
    #[xml(skip_deserializing)]
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
pub struct CalendarResource(Calendar);

impl Resource for CalendarResource {
    type PropName = CalendarPropName;
    type Prop = CalendarProp;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> &'static [&'static str] {
        if self.0.subscription_url.is_none() {
            &["collection", "C:calendar"]
        } else {
            &["collection", "CS:subscribed"]
        }
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        _user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            CalendarPropName::Displayname => CalendarProp::Displayname(self.0.displayname.clone()),
            CalendarPropName::CalendarColor => CalendarProp::CalendarColor(self.0.color.clone()),
            CalendarPropName::CalendarDescription => {
                CalendarProp::CalendarDescription(self.0.description.clone())
            }
            CalendarPropName::CalendarTimezone => {
                CalendarProp::CalendarTimezone(self.0.timezone.clone())
            }
            CalendarPropName::CalendarOrder => CalendarProp::CalendarOrder(Some(self.0.order)),
            CalendarPropName::SupportedCalendarComponentSet => {
                CalendarProp::SupportedCalendarComponentSet(SupportedCalendarComponentSet {
                    comp: vec![
                        SupportedCalendarComponent {
                            name: "VEVENT".to_owned(),
                        },
                        SupportedCalendarComponent {
                            name: "VTODO".to_owned(),
                        },
                        SupportedCalendarComponent {
                            name: "VJOURNAL".to_owned(),
                        },
                    ],
                })
            }
            CalendarPropName::SupportedCalendarData => {
                CalendarProp::SupportedCalendarData(SupportedCalendarData::default())
            }
            CalendarPropName::Getcontenttype => {
                CalendarProp::Getcontenttype("text/calendar;charset=utf-8".to_owned())
            }
            CalendarPropName::Transports => CalendarProp::Transports(Default::default()),
            CalendarPropName::Topic => {
                let url = CalendarResource::get_url(rmap, [&self.0.principal, &self.0.id]).unwrap();
                let mut hasher = Sha256::new();
                hasher.update(url);
                let topic = format!("{:x}", hasher.finalize());
                CalendarProp::Topic(topic)
            }
            CalendarPropName::MaxResourceSize => CalendarProp::MaxResourceSize(10000000),
            CalendarPropName::SupportedReportSet => {
                CalendarProp::SupportedReportSet(SupportedReportSet::default())
            }
            CalendarPropName::SyncToken => CalendarProp::SyncToken(self.0.format_synctoken()),
            CalendarPropName::Getctag => CalendarProp::Getctag(self.0.format_synctoken()),
            CalendarPropName::Source => {
                CalendarProp::Source(self.0.subscription_url.to_owned().map(HrefElement::from))
            }
        })
    }

    fn set_prop(&mut self, prop: Self::Prop) -> Result<(), rustical_dav::Error> {
        match prop {
            CalendarProp::Displayname(displayname) => {
                self.0.displayname = displayname;
                Ok(())
            }
            CalendarProp::CalendarColor(color) => {
                self.0.color = color;
                Ok(())
            }
            CalendarProp::CalendarDescription(description) => {
                self.0.description = description;
                Ok(())
            }
            CalendarProp::CalendarTimezone(timezone) => {
                self.0.timezone = timezone;
                Ok(())
            }
            CalendarProp::CalendarOrder(order) => {
                self.0.order = order.unwrap_or_default();
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
        match prop {
            CalendarPropName::Displayname => {
                self.0.displayname = None;
                Ok(())
            }
            CalendarPropName::CalendarColor => {
                self.0.color = None;
                Ok(())
            }
            CalendarPropName::CalendarDescription => {
                self.0.description = None;
                Ok(())
            }
            CalendarPropName::CalendarTimezone => {
                self.0.timezone = None;
                Ok(())
            }
            CalendarPropName::CalendarOrder => {
                self.0.order = 0;
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

    #[inline]
    fn resource_name() -> &'static str {
        "caldav_calendar"
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.0.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        // TODO: read-only for subscription
        Ok(UserPrivilegeSet::owner_only(self.0.principal == user.id))
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for CalendarResourceService<C> {
    type MemberType = CalendarObjectResource;
    type PathComponents = (String, String); // principal, calendar_id
    type Resource = CalendarResource;
    type Error = Error;

    async fn get_resource(&self) -> Result<Self::Resource, Error> {
        let calendar = self
            .cal_store
            .get_calendar(&self.principal, &self.calendar_id)
            .await
            .map_err(|_e| Error::NotFound)?;
        Ok(calendar.into())
    }

    async fn get_members(
        &self,
        rmap: &ResourceMap,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(self
            .cal_store
            .get_objects(&self.principal, &self.calendar_id)
            .await?
            .into_iter()
            .map(|object| {
                (
                    CalendarObjectResource::get_url(
                        rmap,
                        vec![&self.principal, &self.calendar_id, object.get_id()],
                    )
                    .unwrap(),
                    CalendarObjectResource {
                        object,
                        principal: self.principal.to_owned(),
                    },
                )
            })
            .collect())
    }

    async fn new(
        req: &HttpRequest,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let cal_store = req
            .app_data::<Data<C>>()
            .expect("no calendar store in app_data!")
            .clone()
            .into_inner();

        Ok(Self {
            principal: path_components.0,
            calendar_id: path_components.1,
            cal_store,
        })
    }

    async fn save_resource(&self, file: Self::Resource) -> Result<(), Self::Error> {
        self.cal_store
            .update_calendar(
                self.principal.to_owned(),
                self.calendar_id.to_owned(),
                file.into(),
            )
            .await?;
        Ok(())
    }

    async fn delete_resource(&self, use_trashbin: bool) -> Result<(), Self::Error> {
        self.cal_store
            .delete_calendar(&self.principal, &self.calendar_id, use_trashbin)
            .await?;
        Ok(())
    }

    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        let report_method = web::method(Method::from_str("REPORT").unwrap());
        let mkcalendar_method = web::method(Method::from_str("MKCALENDAR").unwrap());

        res.route(report_method.to(route_report_calendar::<C>))
            .route(mkcalendar_method.to(route_mkcalendar::<C>))
            .post(route_post::<C>)
    }
}
