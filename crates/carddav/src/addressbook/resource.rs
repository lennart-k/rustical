use super::methods::mkcol::route_mkcol;
use super::methods::post::route_post;
use super::methods::report::route_report_addressbook;
use super::prop::{SupportedAddressData, SupportedReportSet};
use crate::address_object::resource::{AddressObjectResource, AddressObjectResourceService};
use crate::{CardDavPrincipalUri, Error};
use actix_web::http::Method;
use actix_web::web;
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, SyncTokenExtension, SyncTokenExtensionProp,
};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{PrincipalUri, Resource, ResourceService};
use rustical_dav::xml::{Resourcetype, ResourcetypeInner};
use rustical_dav_push::{DavPushExtension, DavPushExtensionProp};
use rustical_store::auth::User;
use rustical_store::{Addressbook, AddressbookStore, SubscriptionStore};
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};
use std::str::FromStr;
use std::sync::Arc;

pub struct AddressbookResourceService<AS: AddressbookStore, S: SubscriptionStore> {
    pub(crate) addr_store: Arc<AS>,
    pub(crate) sub_store: Arc<S>,
}

impl<A: AddressbookStore, S: SubscriptionStore> AddressbookResourceService<A, S> {
    pub fn new(addr_store: Arc<A>, sub_store: Arc<S>) -> Self {
        Self {
            addr_store,
            sub_store,
        }
    }
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "AddressbookPropName")]
pub enum AddressbookProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(Option<String>),

    // CardDAV (RFC 6352)
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookDescription(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", skip_deserializing)]
    SupportedAddressData(SupportedAddressData),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", skip_deserializing)]
    SupportedReportSet(SupportedReportSet),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    MaxResourceSize(i64),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "AddressbookPropWrapperName", untagged)]
pub enum AddressbookPropWrapper {
    Addressbook(AddressbookProp),
    SyncToken(SyncTokenExtensionProp),
    DavPush(DavPushExtensionProp),
    Common(CommonPropertiesProp),
}

#[derive(Clone, Debug, From, Into)]
pub struct AddressbookResource(pub(crate) Addressbook);

impl SyncTokenExtension for AddressbookResource {
    fn get_synctoken(&self) -> String {
        self.0.format_synctoken()
    }
}

impl DavPushExtension for AddressbookResource {
    fn get_topic(&self) -> String {
        self.0.push_topic.to_owned()
    }
}

impl Resource for AddressbookResource {
    type Prop = AddressbookPropWrapper;
    type Error = Error;
    type Principal = User;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "collection"),
            ResourcetypeInner(Some(rustical_dav::namespace::NS_CARDDAV), "addressbook"),
        ])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &User,
        prop: &AddressbookPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            AddressbookPropWrapperName::Addressbook(prop) => {
                AddressbookPropWrapper::Addressbook(match prop {
                    AddressbookPropName::Displayname => {
                        AddressbookProp::Displayname(self.0.displayname.clone())
                    }
                    AddressbookPropName::MaxResourceSize => {
                        AddressbookProp::MaxResourceSize(10000000)
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
                })
            }

            AddressbookPropWrapperName::SyncToken(prop) => AddressbookPropWrapper::SyncToken(
                <Self as SyncTokenExtension>::get_prop(self, prop)?,
            ),
            AddressbookPropWrapperName::DavPush(prop) => {
                AddressbookPropWrapper::DavPush(<Self as DavPushExtension>::get_prop(self, prop)?)
            }
            AddressbookPropWrapperName::Common(prop) => AddressbookPropWrapper::Common(
                CommonPropertiesExtension::get_prop(self, puri, user, prop)?,
            ),
        })
    }

    fn set_prop(&mut self, prop: Self::Prop) -> Result<(), rustical_dav::Error> {
        match prop {
            AddressbookPropWrapper::Addressbook(prop) => match prop {
                AddressbookProp::Displayname(displayname) => {
                    self.0.displayname = displayname;
                    Ok(())
                }
                AddressbookProp::AddressbookDescription(description) => {
                    self.0.description = description;
                    Ok(())
                }
                AddressbookProp::MaxResourceSize(_) => Err(rustical_dav::Error::PropReadOnly),
                AddressbookProp::SupportedReportSet(_) => Err(rustical_dav::Error::PropReadOnly),
                AddressbookProp::SupportedAddressData(_) => Err(rustical_dav::Error::PropReadOnly),
            },
            AddressbookPropWrapper::SyncToken(prop) => SyncTokenExtension::set_prop(self, prop),
            AddressbookPropWrapper::DavPush(prop) => DavPushExtension::set_prop(self, prop),
            AddressbookPropWrapper::Common(prop) => CommonPropertiesExtension::set_prop(self, prop),
        }
    }

    fn remove_prop(
        &mut self,
        prop: &AddressbookPropWrapperName,
    ) -> Result<(), rustical_dav::Error> {
        match prop {
            AddressbookPropWrapperName::Addressbook(prop) => match prop {
                AddressbookPropName::Displayname => {
                    self.0.displayname = None;
                    Ok(())
                }
                AddressbookPropName::AddressbookDescription => {
                    self.0.description = None;
                    Ok(())
                }
                AddressbookPropName::MaxResourceSize => Err(rustical_dav::Error::PropReadOnly),
                AddressbookPropName::SupportedReportSet => Err(rustical_dav::Error::PropReadOnly),
                AddressbookPropName::SupportedAddressData => Err(rustical_dav::Error::PropReadOnly),
            },
            AddressbookPropWrapperName::SyncToken(prop) => {
                SyncTokenExtension::remove_prop(self, prop)
            }
            AddressbookPropWrapperName::DavPush(prop) => DavPushExtension::remove_prop(self, prop),
            AddressbookPropWrapperName::Common(prop) => {
                CommonPropertiesExtension::remove_prop(self, prop)
            }
        }
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.0.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.0.principal),
        ))
    }
}

#[async_trait]
impl<AS: AddressbookStore, S: SubscriptionStore> ResourceService
    for AddressbookResourceService<AS, S>
{
    type MemberType = AddressObjectResource;
    type PathComponents = (String, String); // principal, addressbook_id
    type Resource = AddressbookResource;
    type Error = Error;
    type Principal = User;
    type PrincipalUri = CardDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, addressbook";

    async fn get_resource(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
    ) -> Result<Self::Resource, Error> {
        let addressbook = self
            .addr_store
            .get_addressbook(principal, addressbook_id, false)
            .await
            .map_err(|_e| Error::NotFound)?;
        Ok(addressbook.into())
    }

    async fn get_members(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(self
            .addr_store
            .get_objects(principal, addressbook_id)
            .await?
            .into_iter()
            .map(|object| {
                (
                    format!("{}.vcf", object.get_id()),
                    AddressObjectResource {
                        object,
                        principal: principal.to_owned(),
                    },
                )
            })
            .collect())
    }

    async fn save_resource(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
        file: Self::Resource,
    ) -> Result<(), Self::Error> {
        self.addr_store
            .update_addressbook(principal.to_owned(), addressbook_id.to_owned(), file.into())
            .await?;
        Ok(())
    }

    async fn delete_resource(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
        use_trashbin: bool,
    ) -> Result<(), Self::Error> {
        self.addr_store
            .delete_addressbook(principal, addressbook_id, use_trashbin)
            .await?;
        Ok(())
    }

    #[inline]
    fn actix_scope(self) -> actix_web::Scope {
        let mkcol_method = web::method(Method::from_str("MKCOL").unwrap());
        let report_method = web::method(Method::from_str("REPORT").unwrap());
        web::scope("/{addressbook_id}")
            .service(AddressObjectResourceService::<AS>::new(self.addr_store.clone()).actix_scope())
            .service(
                self.actix_resource()
                    .route(mkcol_method.to(route_mkcol::<AS>))
                    .route(report_method.to(route_report_addressbook::<AS>))
                    .post(route_post::<AS, S>),
            )
    }
}
