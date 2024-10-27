use crate::calendar::resource::CalendarResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::web::Data;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::CalendarStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, VariantNames};

pub struct PrincipalResourceService<C: CalendarStore + ?Sized> {
    principal: String,
    cal_store: Arc<C>,
}

#[derive(Clone)]
pub struct PrincipalResource {
    principal: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    principal: (),
    collection: (),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PrincipalProp {
    // WebDAV (RFC 2518)
    Resourcetype(Resourcetype),

    // WebDAV Access Control (RFC 3744)
    #[serde(rename = "principal-URL")]
    PrincipalUrl(HrefElement),

    // WebDAV Current Principal Extension (RFC 5397)
    CurrentUserPrincipal(HrefElement),

    // CalDAV (RFC 4791)
    #[serde(rename = "C:calendar-home-set")]
    CalendarHomeSet(HrefElement),
    #[serde(rename = "C:calendar-user-address-set")]
    CalendarUserAddressSet(HrefElement),

    #[serde(other)]
    Invalid,
}

impl InvalidProperty for PrincipalProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(EnumString, Debug, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum PrincipalPropName {
    Resourcetype,
    CurrentUserPrincipal,
    #[strum(serialize = "principal-URL")]
    PrincipalUrl,
    CalendarHomeSet,
    CalendarUserAddressSet,
}

impl Resource for PrincipalResource {
    type PropName = PrincipalPropName;
    type Prop = PrincipalProp;
    type Error = Error;

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        prop: Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(Self::get_url(rmap, vec![&self.principal]).unwrap());

        Ok(match prop {
            PrincipalPropName::Resourcetype => PrincipalProp::Resourcetype(Resourcetype::default()),
            PrincipalPropName::CurrentUserPrincipal => {
                PrincipalProp::CurrentUserPrincipal(principal_href)
            }
            PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_href),
            PrincipalPropName::CalendarHomeSet => PrincipalProp::CalendarHomeSet(principal_href),
            PrincipalPropName::CalendarUserAddressSet => {
                PrincipalProp::CalendarUserAddressSet(principal_href)
            }
        })
    }

    #[inline]
    fn resource_name() -> &'static str {
        "caldav_principal"
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for PrincipalResourceService<C> {
    type PathComponents = (String,);
    type MemberType = CalendarResource;
    type Resource = PrincipalResource;
    type Error = Error;

    async fn new(
        req: &HttpRequest,
        (principal,): Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let cal_store = req
            .app_data::<Data<C>>()
            .expect("no calendar store in app_data!")
            .clone()
            .into_inner();

        Ok(Self {
            cal_store,
            principal,
        })
    }

    async fn get_resource(&self, principal: String) -> Result<Self::Resource, Self::Error> {
        if self.principal != principal {
            return Err(Error::Unauthorized);
        }
        Ok(PrincipalResource {
            principal: self.principal.to_owned(),
        })
    }

    async fn get_members(
        &self,
        rmap: &ResourceMap,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        let calendars = self.cal_store.get_calendars(&self.principal).await?;
        Ok(calendars
            .into_iter()
            .map(|cal| {
                (
                    CalendarResource::get_url(rmap, vec![&self.principal, &cal.id]).unwrap(),
                    cal.into(),
                )
            })
            .collect())
    }

    async fn save_resource(&self, _file: Self::Resource) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }
}
