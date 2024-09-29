use crate::Error;
use actix_web::web::Data;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::CalendarStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, VariantNames};
use tokio::sync::RwLock;

use crate::calendar::resource::CalendarResource;

pub struct PrincipalResourceService<C: CalendarStore + ?Sized> {
    principal: String,
    path: String,
    cal_store: Arc<RwLock<C>>,
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
    Resourcetype(Resourcetype),
    CurrentUserPrincipal(HrefElement),
    #[serde(rename = "principal-URL")]
    PrincipalUrl(HrefElement),
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

    fn get_prop(&self, prefix: &str, prop: Self::PropName) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(format!("{}/user/{}/", prefix, self.principal));
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
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for PrincipalResourceService<C> {
    type PathComponents = (String,);
    type MemberType = CalendarResource;
    type File = PrincipalResource;
    type Error = Error;

    async fn new(
        req: &HttpRequest,
        auth_info: &AuthInfo,
        (principal,): Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        if auth_info.user_id != principal {
            return Err(Error::Unauthorized);
        }
        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .expect("no calendar store in app_data!")
            .clone()
            .into_inner();

        Ok(Self {
            cal_store,
            path: req.path().to_owned(),
            principal,
        })
    }

    async fn get_file(&self) -> Result<Self::File, Self::Error> {
        Ok(PrincipalResource {
            principal: self.principal.to_owned(),
        })
    }

    async fn get_members(
        &self,
        _auth_info: AuthInfo,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        let calendars = self
            .cal_store
            .read()
            .await
            .get_calendars(&self.principal)
            .await?;
        Ok(calendars
            .into_iter()
            .map(|cal| (format!("{}/{}", &self.path, &cal.id), cal.into()))
            .collect())
    }

    async fn save_file(&self, _file: Self::File) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }
}
