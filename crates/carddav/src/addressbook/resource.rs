use super::methods::mkcol::route_mkcol;
use super::methods::report::route_report_addressbook;
use super::prop::{Resourcetype, SupportedAddressData, SupportedReportSet, UserPrivilegeSet};
use crate::address_object::resource::AddressObjectResource;
use crate::principal::PrincipalResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::http::Method;
use actix_web::web;
use actix_web::{web::Data, HttpRequest};
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::model::Addressbook;
use rustical_store::AddressbookStore;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use strum::{EnumString, VariantNames};

pub struct AddressbookResourceService<AS: AddressbookStore + ?Sized> {
    pub addr_store: Arc<AS>,
    pub path: String,
    pub principal: String,
    pub addressbook_id: String,
}

#[derive(EnumString, Debug, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum AddressbookPropName {
    Resourcetype,
    Displayname,
    Getcontenttype,
    CurrentUserPrincipal,
    Owner,
    CurrentUserPrivilegeSet,
    AddressbookDescription,
    SupportedAddressData,
    SupportedReportSet,
    MaxResourceSize,
    SyncToken,
    Getctag,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AddressbookProp {
    // WebDAV (RFC 2518)
    Resourcetype(Resourcetype),
    Displayname(Option<String>),
    Getcontenttype(String),

    // WebDAV Current Principal Extension (RFC 5397)
    CurrentUserPrincipal(HrefElement),

    // WebDAV Access Control (RFC 3744)
    Owner(HrefElement),
    CurrentUserPrivilegeSet(UserPrivilegeSet),

    // CardDAV (RFC 6352)
    #[serde(
        rename = "CARD:addressbook-description",
        alias = "addressbook-description"
    )]
    AddressbookDescription(Option<String>),
    #[serde(
        rename = "CARD:supported-address-data",
        alias = "supported-address-data"
    )]
    SupportedAddressData(SupportedAddressData),
    SupportedReportSet(SupportedReportSet),
    MaxResourceSize(i64),

    // Collection Synchronization (RFC 6578)
    SyncToken(String),

    // Didn't find the spec
    Getctag(String),

    #[serde(other)]
    Invalid,
}

impl InvalidProperty for AddressbookProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(Clone, Debug, From, Into)]
pub struct AddressbookResource(Addressbook);

impl Resource for AddressbookResource {
    type PropName = AddressbookPropName;
    type Prop = AddressbookProp;
    type Error = Error;

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        prop: Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            AddressbookPropName::Resourcetype => {
                AddressbookProp::Resourcetype(Resourcetype::default())
            }
            AddressbookPropName::CurrentUserPrincipal => {
                AddressbookProp::CurrentUserPrincipal(HrefElement::new(
                    PrincipalResource::get_url(rmap, vec![&self.0.principal]).unwrap(),
                ))
            }
            AddressbookPropName::Owner => AddressbookProp::Owner(HrefElement::new(
                PrincipalResource::get_url(rmap, vec![&self.0.principal]).unwrap(),
            )),
            AddressbookPropName::Displayname => {
                AddressbookProp::Displayname(self.0.displayname.clone())
            }
            AddressbookPropName::Getcontenttype => {
                AddressbookProp::Getcontenttype("text/vcard;charset=utf-8".to_owned())
            }
            AddressbookPropName::MaxResourceSize => AddressbookProp::MaxResourceSize(10000000),
            AddressbookPropName::CurrentUserPrivilegeSet => {
                AddressbookProp::CurrentUserPrivilegeSet(UserPrivilegeSet::default())
            }
            AddressbookPropName::SupportedReportSet => {
                AddressbookProp::SupportedReportSet(SupportedReportSet::default())
            }
            AddressbookPropName::AddressbookDescription => {
                AddressbookProp::AddressbookDescription(self.0.description.to_owned())
            }
            AddressbookPropName::SupportedAddressData => {
                AddressbookProp::SupportedAddressData(SupportedAddressData::default())
            }
            AddressbookPropName::SyncToken => AddressbookProp::SyncToken(self.0.format_synctoken()),
            AddressbookPropName::Getctag => AddressbookProp::Getctag(self.0.format_synctoken()),
        })
    }

    fn set_prop(&mut self, prop: Self::Prop) -> Result<(), rustical_dav::Error> {
        match prop {
            AddressbookProp::Resourcetype(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::CurrentUserPrincipal(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::Owner(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::Displayname(displayname) => {
                self.0.displayname = displayname;
                Ok(())
            }
            AddressbookProp::AddressbookDescription(description) => {
                self.0.description = description;
                Ok(())
            }
            AddressbookProp::Getcontenttype(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::MaxResourceSize(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::CurrentUserPrivilegeSet(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::SupportedReportSet(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::SupportedAddressData(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::SyncToken(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::Getctag(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::Invalid => Err(rustical_dav::Error::PropReadOnly),
        }
    }

    fn remove_prop(&mut self, prop: Self::PropName) -> Result<(), rustical_dav::Error> {
        match prop {
            AddressbookPropName::Resourcetype => Err(rustical_dav::Error::PropReadOnly),
            AddressbookPropName::CurrentUserPrincipal => Err(rustical_dav::Error::PropReadOnly),
            AddressbookPropName::Owner => Err(rustical_dav::Error::PropReadOnly),
            AddressbookPropName::Displayname => {
                self.0.displayname = None;
                Ok(())
            }
            AddressbookPropName::AddressbookDescription => {
                self.0.description = None;
                Ok(())
            }
            AddressbookPropName::Getcontenttype => Err(rustical_dav::Error::PropReadOnly),
            AddressbookPropName::MaxResourceSize => Err(rustical_dav::Error::PropReadOnly),
            AddressbookPropName::CurrentUserPrivilegeSet => Err(rustical_dav::Error::PropReadOnly),
            AddressbookPropName::SupportedReportSet => Err(rustical_dav::Error::PropReadOnly),
            AddressbookPropName::SupportedAddressData => Err(rustical_dav::Error::PropReadOnly),
            AddressbookPropName::SyncToken => Err(rustical_dav::Error::PropReadOnly),
            AddressbookPropName::Getctag => Err(rustical_dav::Error::PropReadOnly),
        }
    }

    #[inline]
    fn resource_name() -> &'static str {
        "carddav_addressbook"
    }
}

#[async_trait(?Send)]
impl<AS: AddressbookStore + ?Sized> ResourceService for AddressbookResourceService<AS> {
    type MemberType = AddressObjectResource;
    type PathComponents = (String, String); // principal, addressbook_id
    type Resource = AddressbookResource;
    type Error = Error;

    async fn get_resource(&self, principal: String) -> Result<Self::Resource, Error> {
        if self.principal != principal {
            return Err(Error::Unauthorized);
        }
        let addressbook = self
            .addr_store
            .get_addressbook(&self.principal, &self.addressbook_id)
            .await
            .map_err(|_e| Error::NotFound)?;
        Ok(addressbook.into())
    }

    async fn get_members(
        &self,
        rmap: &ResourceMap,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(self
            .addr_store
            .get_objects(&self.principal, &self.addressbook_id)
            .await?
            .into_iter()
            .map(|object| {
                (
                    AddressObjectResource::get_url(
                        rmap,
                        vec![&self.principal, &self.addressbook_id, object.get_id()],
                    )
                    .unwrap(),
                    object.into(),
                )
            })
            .collect())
    }

    async fn new(
        req: &HttpRequest,
        path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let addr_store = req
            .app_data::<Data<AS>>()
            .expect("no addressbook store in app_data!")
            .clone()
            .into_inner();

        Ok(Self {
            path: req.path().to_owned(),
            principal: path_components.0,
            addressbook_id: path_components.1,
            addr_store,
        })
    }

    async fn save_resource(&self, file: Self::Resource) -> Result<(), Self::Error> {
        self.addr_store
            .update_addressbook(
                self.principal.to_owned(),
                self.addressbook_id.to_owned(),
                file.into(),
            )
            .await?;
        Ok(())
    }

    async fn delete_resource(&self, use_trashbin: bool) -> Result<(), Self::Error> {
        self.addr_store
            .delete_addressbook(&self.principal, &self.addressbook_id, use_trashbin)
            .await?;
        Ok(())
    }

    #[inline]
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        let mkcol_method = web::method(Method::from_str("MKCOL").unwrap());
        let report_method = web::method(Method::from_str("REPORT").unwrap());
        res.route(mkcol_method.to(route_mkcol::<AS>))
            .route(report_method.to(route_report_addressbook::<AS>))
    }
}