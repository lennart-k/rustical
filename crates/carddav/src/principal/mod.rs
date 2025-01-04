use crate::addressbook::resource::AddressbookResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use async_trait::async_trait;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_store::auth::User;
use rustical_store::AddressbookStore;
use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::sync::Arc;
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantNames};

pub struct PrincipalResourceService<A: AddressbookStore + ?Sized> {
    addr_store: Arc<A>,
}

impl<A: AddressbookStore + ?Sized> PrincipalResourceService<A> {
    pub fn new(addr_store: Arc<A>) -> Self {
        Self { addr_store }
    }
}

#[derive(Debug, Clone)]
pub struct PrincipalResource {
    principal: String,
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, EnumDiscriminants, Clone)]
#[strum_discriminants(
    name(PrincipalPropName),
    derive(EnumString, VariantNames, IntoStaticStr),
    strum(serialize_all = "kebab-case")
)]
pub enum PrincipalProp {
    // WebDAV Access Control (RFC 3744)
    #[strum_discriminants(strum(serialize = "principal-URL"))]
    #[xml(rename = b"principal-URL")]
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    PrincipalUrl(HrefElement),

    // CardDAV (RFC 6352)
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookHomeSet(HrefElement),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    PrincipalAddress(Option<HrefElement>),
}

impl PrincipalResource {
    pub fn get_principal_url(rmap: &ResourceMap, principal: &str) -> String {
        Self::get_url(rmap, vec![principal]).unwrap()
    }
}

impl Resource for PrincipalResource {
    type PropName = PrincipalPropName;
    type Prop = PrincipalProp;
    type Error = Error;
    type PrincipalResource = PrincipalResource;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype {
            inner: &[
                ResourcetypeInner {
                    ns: rustical_dav::namespace::NS_DAV,
                    name: "collection",
                },
                ResourcetypeInner {
                    ns: rustical_dav::namespace::NS_DAV,
                    name: "principal",
                },
            ],
        }
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        _user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(Self::get_principal_url(rmap, &self.principal));

        Ok(match prop {
            PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_href),
            PrincipalPropName::AddressbookHomeSet => {
                PrincipalProp::AddressbookHomeSet(principal_href)
            }
            PrincipalPropName::PrincipalAddress => PrincipalProp::PrincipalAddress(None),
        })
    }

    #[inline]
    fn resource_name() -> &'static str {
        "carddav_principal"
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(self.principal == user.id))
    }
}

#[async_trait(?Send)]
impl<A: AddressbookStore + ?Sized> ResourceService for PrincipalResourceService<A> {
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
        rmap: &ResourceMap,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        let addressbooks = self.addr_store.get_addressbooks(principal).await?;
        Ok(addressbooks
            .into_iter()
            .map(|addressbook| {
                (
                    AddressbookResource::get_url(rmap, vec![principal, &addressbook.id]).unwrap(),
                    addressbook.into(),
                )
            })
            .collect())
    }
}
