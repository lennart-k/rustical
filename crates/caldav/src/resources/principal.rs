use crate::tagname::TagName;
use actix_web::{web::Data, HttpRequest};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::{
    resource::Resource,
    xml_snippets::{write_resourcetype, HrefElement},
};
use rustical_store::calendar::CalendarStore;
use std::sync::Arc;
use strum::{EnumProperty, EnumString, IntoStaticStr, VariantNames};
use tokio::sync::RwLock;

use super::calendar::CalendarResource;

pub struct PrincipalCalendarsResource<C: CalendarStore + ?Sized> {
    prefix: String,
    principal: String,
    path: String,
    cal_store: Arc<RwLock<C>>,
}

#[derive(EnumString, Debug, VariantNames, IntoStaticStr, EnumProperty)]
#[strum(serialize_all = "kebab-case")]
pub enum PrincipalProp {
    Resourcetype,
    CurrentUserPrincipal,
    #[strum(serialize = "principal-URL")]
    PrincipalUrl,
    #[strum(props(tagname = "C:calendar-home-set"))]
    CalendarHomeSet,
    #[strum(props(tagname = "C:calendar-user-address-set"))]
    CalendarUserAddressSet,
}

#[async_trait(?Send)]
impl<C: CalendarStore + ?Sized> Resource for PrincipalCalendarsResource<C> {
    type UriComponents = ();
    type MemberType = CalendarResource<C>;
    type PropType = PrincipalProp;

    fn get_path(&self) -> &str {
        &self.path
    }

    async fn get_members(&self) -> Result<Vec<Self::MemberType>> {
        let calendars = self
            .cal_store
            .read()
            .await
            .get_calendars(&self.principal)
            .await?;
        let mut out = Vec::new();
        for calendar in calendars {
            let path = format!("{}/{}", &self.path, &calendar.id);
            out.push(CalendarResource {
                cal_store: self.cal_store.clone(),
                calendar,
                path,
                prefix: self.prefix.clone(),
                principal: self.principal.clone(),
            })
        }
        Ok(out)
    }

    async fn acquire_from_request(
        req: HttpRequest,
        auth_info: AuthInfo,
        _uri_components: Self::UriComponents,
        prefix: String,
    ) -> Result<Self> {
        let cal_store = req
            .app_data::<Data<RwLock<C>>>()
            .ok_or(anyhow!("no calendar store in app_data!"))?
            .clone()
            .into_inner();
        Ok(Self {
            cal_store,
            prefix,
            principal: auth_info.user_id,
            path: req.path().to_string(),
        })
    }

    fn write_prop<W: std::io::Write>(
        &self,
        writer: &mut quick_xml::Writer<W>,
        prop: Self::PropType,
    ) -> Result<()> {
        match prop {
            PrincipalProp::Resourcetype => {
                write_resourcetype(writer, vec!["principal", "collection"])?
            }
            PrincipalProp::CurrentUserPrincipal
            | PrincipalProp::PrincipalUrl
            | PrincipalProp::CalendarHomeSet
            | PrincipalProp::CalendarUserAddressSet => {
                writer.write_serializable(
                    prop.tagname(),
                    &HrefElement::new(format!("{}/{}/", self.prefix, self.principal)),
                )?;
            }
        };
        Ok(())
    }
}
