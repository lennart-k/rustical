use super::methods::mkcol::route_mkcol;
use super::methods::report::route_report_addressbook;
use super::prop::{SupportedAddressData, SupportedReportSet};
use crate::address_object::resource::AddressObjectResource;
use crate::principal::PrincipalResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::http::Method;
use actix_web::web;
use async_trait::async_trait;
use derive_more::derive::{From, Into};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml::{Resourcetype, ResourcetypeInner};
use rustical_store::auth::User;
use rustical_store::{Addressbook, AddressbookStore};
use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::str::FromStr;
use std::sync::Arc;
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantNames};

pub struct AddressbookResourceService<AS: AddressbookStore + ?Sized> {
    addr_store: Arc<AS>,
}

impl<A: AddressbookStore + ?Sized> AddressbookResourceService<A> {
    pub fn new(addr_store: Arc<A>) -> Self {
        Self { addr_store }
    }
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, EnumDiscriminants, Clone)]
#[strum_discriminants(
    name(AddressbookPropName),
    derive(EnumString, VariantNames, IntoStaticStr),
    strum(serialize_all = "kebab-case")
)]
pub enum AddressbookProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", skip_deserializing)]
    Getcontenttype(&'static str),

    // CardDAV (RFC 6352)
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookDescription(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", skip_deserializing)]
    SupportedAddressData(SupportedAddressData),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", skip_deserializing)]
    SupportedReportSet(SupportedReportSet),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    MaxResourceSize(i64),

    // Collection Synchronization (RFC 6578)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SyncToken(String),

    // https://github.com/apple/ccs-calendarserver/blob/master/doc/Extensions/caldav-ctag.txt
    #[xml(ns = "rustical_dav::namespace::NS_CALENDARSERVER")]
    Getctag(String),
}

#[derive(Clone, Debug, From, Into)]
pub struct AddressbookResource(Addressbook);

impl Resource for AddressbookResource {
    type PropName = AddressbookPropName;
    type Prop = AddressbookProp;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[
            ResourcetypeInner {
                ns: rustical_dav::namespace::NS_DAV,
                name: "collection",
            },
            ResourcetypeInner {
                ns: rustical_dav::namespace::NS_CARDDAV,
                name: "addressbook",
            },
        ])
    }

    fn get_prop(
        &self,
        _rmap: &ResourceMap,
        _user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            AddressbookPropName::Displayname => {
                AddressbookProp::Displayname(self.0.displayname.clone())
            }
            AddressbookPropName::Getcontenttype => {
                AddressbookProp::Getcontenttype("text/vcard;charset=utf-8")
            }
            AddressbookPropName::MaxResourceSize => AddressbookProp::MaxResourceSize(10000000),
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
            AddressbookProp::SupportedReportSet(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::SupportedAddressData(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::SyncToken(_) => Err(rustical_dav::Error::PropReadOnly),
            AddressbookProp::Getctag(_) => Err(rustical_dav::Error::PropReadOnly),
        }
    }

    fn remove_prop(&mut self, prop: &Self::PropName) -> Result<(), rustical_dav::Error> {
        match prop {
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

    fn get_owner(&self) -> Option<&str> {
        Some(&self.0.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(self.0.principal == user.id))
    }
}

#[async_trait(?Send)]
impl<AS: AddressbookStore + ?Sized> ResourceService for AddressbookResourceService<AS> {
    type MemberType = AddressObjectResource;
    type PathComponents = (String, String); // principal, addressbook_id
    type Resource = AddressbookResource;
    type Error = Error;

    async fn get_resource(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
    ) -> Result<Self::Resource, Error> {
        let addressbook = self
            .addr_store
            .get_addressbook(principal, addressbook_id)
            .await
            .map_err(|_e| Error::NotFound)?;
        Ok(addressbook.into())
    }

    async fn get_members(
        &self,
        (principal, addressbook_id): &Self::PathComponents,
        rmap: &ResourceMap,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        Ok(self
            .addr_store
            .get_objects(principal, addressbook_id)
            .await?
            .into_iter()
            .map(|object| {
                (
                    AddressObjectResource::get_url(
                        rmap,
                        vec![principal, addressbook_id, object.get_id()],
                    )
                    .unwrap(),
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
    fn actix_additional_routes(res: actix_web::Resource) -> actix_web::Resource {
        let mkcol_method = web::method(Method::from_str("MKCOL").unwrap());
        let report_method = web::method(Method::from_str("REPORT").unwrap());
        res.route(mkcol_method.to(route_mkcol::<AS>))
            .route(report_method.to(route_report_addressbook::<AS>))
    }
}
