use std::sync::Arc;

use actix_web::web::Data;
use actix_web::HttpRequest;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml_snippets::HrefElement;
use rustical_store::calendar::CalendarStore;
use serde::Serialize;
use strum::{EnumString, IntoStaticStr, VariantNames};
use tokio::sync::RwLock;

use super::calendar::CalendarFile;

pub struct PrincipalResource<C: CalendarStore + ?Sized> {
    principal: String,
    path: String,
    cal_store: Arc<RwLock<C>>,
}

pub struct PrincipalFile {
    principal: String,
    path: String,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    principal: (),
    collection: (),
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrincipalPropResponse {
    Resourcetype(Resourcetype),
    CurrentUserPrincipal(HrefElement),
    #[serde(rename = "principal-URL")]
    PrincipalUrl(HrefElement),
    #[serde(rename = "C:calendar-home-set")]
    CalendarHomeSet(HrefElement),
    #[serde(rename = "C:calendar-user-address-set")]
    CalendarUserAddressSet(HrefElement),
}

#[derive(EnumString, Debug, VariantNames, IntoStaticStr, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum PrincipalProp {
    Resourcetype,
    CurrentUserPrincipal,
    #[strum(serialize = "principal-URL")]
    PrincipalUrl,
    CalendarHomeSet,
    CalendarUserAddressSet,
}

#[async_trait(?Send)]
impl Resource for PrincipalFile {
    type PropType = PrincipalProp;
    type PropResponse = PrincipalPropResponse;

    fn get_prop(&self, prefix: &str, prop: Self::PropType) -> Result<Self::PropResponse> {
        match prop {
            PrincipalProp::Resourcetype => {
                Ok(PrincipalPropResponse::Resourcetype(Resourcetype::default()))
            }
            PrincipalProp::CurrentUserPrincipal => Ok(PrincipalPropResponse::CurrentUserPrincipal(
                HrefElement::new(format!("{}/{}/", prefix, self.principal)),
            )),
            PrincipalProp::PrincipalUrl => Ok(PrincipalPropResponse::PrincipalUrl(
                HrefElement::new(format!("{}/{}/", prefix, self.principal)),
            )),
            PrincipalProp::CalendarHomeSet => Ok(PrincipalPropResponse::CalendarHomeSet(
                HrefElement::new(format!("{}/{}/", prefix, self.principal)),
            )),
            PrincipalProp::CalendarUserAddressSet => {
                Ok(PrincipalPropResponse::CalendarUserAddressSet(
                    HrefElement::new(format!("{}/{}/", prefix, self.principal)),
                ))
            }
        }
    }

    fn get_path(&self) -> &str {
        &self.path
    }
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> ResourceService for PrincipalResource<C> {
    type PathComponents = (String,);
    type MemberType = CalendarFile;
    type File = PrincipalFile;

    async fn new(
        req: HttpRequest,
        auth_info: AuthInfo,
        _path_components: Self::PathComponents,
    ) -> Result<Self, rustical_dav::error::Error> {
        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();

        Ok(Self {
            cal_store,
            path: req.path().to_owned(),
            principal: auth_info.user_id,
        })
    }

    async fn get_file(&self) -> Result<Self::File> {
        Ok(PrincipalFile {
            principal: self.principal.to_owned(),
            path: self.path.to_owned(),
        })
    }

    async fn get_members(&self, _auth_info: AuthInfo) -> Result<Vec<Self::MemberType>> {
        let calendars = self
            .cal_store
            .read()
            .await
            .get_calendars(&self.principal)
            .await?;
        Ok(calendars
            .into_iter()
            .map(|cal| CalendarFile {
                calendar: cal,
                principal: self.principal.to_owned(),
                path: self.path.to_owned(),
            })
            .collect())
    }
}
