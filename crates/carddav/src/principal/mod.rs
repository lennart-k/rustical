use crate::addressbook::resource::AddressbookResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::web::Data;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::auth::User;
use rustical_store::AddressbookStore;
use rustical_xml::{XmlDeserialize, XmlSerialize};
use std::sync::Arc;
use strum::{EnumDiscriminants, EnumString, IntoStaticStr, VariantNames};

pub struct PrincipalResourceService<A: AddressbookStore + ?Sized> {
    principal: String,
    addr_store: Arc<A>,
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

    fn get_resourcetype(&self) -> &'static [&'static str] {
        &["collection", "principal"]
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

    async fn new(
        req: &HttpRequest,
        (principal,): Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        let addr_store = req
            .app_data::<Data<A>>()
            .expect("no addressbook store in app_data!")
            .clone()
            .into_inner();

        Ok(Self {
            addr_store,
            principal,
        })
    }

    async fn get_resource(&self) -> Result<Self::Resource, Self::Error> {
        Ok(PrincipalResource {
            principal: self.principal.to_owned(),
        })
    }

    async fn get_members(
        &self,
        rmap: &ResourceMap,
    ) -> Result<Vec<(String, Self::MemberType)>, Self::Error> {
        let addressbooks = self.addr_store.get_addressbooks(&self.principal).await?;
        Ok(addressbooks
            .into_iter()
            .map(|addressbook| {
                (
                    AddressbookResource::get_url(rmap, vec![&self.principal, &addressbook.id])
                        .unwrap(),
                    addressbook.into(),
                )
            })
            .collect())
    }
}
