use crate::addressbook::resource::AddressbookResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use async_trait::async_trait;
use rustical_dav::extensions::{CommonPropertiesExtension, CommonPropertiesProp};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{NamedRoute, Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_store::auth::User;
use rustical_store::AddressbookStore;
use rustical_xml::{EnumUnitVariants, EnumVariants, XmlDeserialize, XmlSerialize};
use std::sync::Arc;

pub struct PrincipalResourceService<A: AddressbookStore> {
    addr_store: Arc<A>,
}

impl<A: AddressbookStore> PrincipalResourceService<A> {
    pub fn new(addr_store: Arc<A>) -> Self {
        Self { addr_store }
    }
}

#[derive(Debug, Clone)]
pub struct PrincipalResource {
    principal: String,
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "PrincipalPropName")]
pub enum PrincipalProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(String),

    // WebDAV Access Control (RFC 3744)
    #[xml(rename = b"principal-URL")]
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    PrincipalUrl(HrefElement),

    // CardDAV (RFC 6352)
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookHomeSet(HrefElement),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    PrincipalAddress(Option<HrefElement>),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "PrincipalPropWrapperName", untagged)]
pub enum PrincipalPropWrapper {
    Principal(PrincipalProp),
    Common(CommonPropertiesProp),
}

impl PrincipalResource {
    pub fn get_principal_url(rmap: &ResourceMap, principal: &str) -> String {
        Self::get_url(rmap, vec![principal]).unwrap()
    }
}

impl NamedRoute for PrincipalResource {
    fn route_name() -> &'static str {
        "carddav_principal"
    }
}

impl Resource for PrincipalResource {
    type Prop = PrincipalPropWrapper;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "collection"),
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "principal"),
        ])
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        user: &User,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(Self::get_principal_url(rmap, &self.principal));

        Ok(match prop {
            PrincipalPropWrapperName::Principal(prop) => {
                PrincipalPropWrapper::Principal(match prop {
                    PrincipalPropName::Displayname => {
                        PrincipalProp::Displayname(self.principal.to_owned())
                    }
                    PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_href),
                    PrincipalPropName::AddressbookHomeSet => {
                        PrincipalProp::AddressbookHomeSet(principal_href)
                    }
                    PrincipalPropName::PrincipalAddress => PrincipalProp::PrincipalAddress(None),
                })
            }

            PrincipalPropWrapperName::Common(prop) => PrincipalPropWrapper::Common(
                CommonPropertiesExtension::get_prop(self, rmap, user, prop)?,
            ),
        })
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(self.principal == user.id))
    }
}

#[async_trait(?Send)]
impl<A: AddressbookStore> ResourceService for PrincipalResourceService<A> {
    type PathComponents = (String,);
    type MemberType = AddressbookResource;
    type Resource = PrincipalResource;
    type Error = Error;

    async fn get_resource(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        Ok(PrincipalResource {
            principal: principal.to_owned(),
        })
    }

    async fn get_members(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        let addressbooks = self.addr_store.get_addressbooks(principal).await?;
        Ok(addressbooks
            .into_iter()
            .map(|addressbook| (addressbook.id.to_owned(), addressbook.into()))
            .collect())
    }
}
