use crate::addressbook::resource::AddressbookResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::web::Data;
use actix_web::HttpRequest;
use async_trait::async_trait;
use derive_more::derive::{From, TryInto};
use rustical_dav::extension::BoxedExtension;
use rustical_dav::extensions::{
    CommonPropertiesExtension, CommonPropertiesProp, CommonPropertiesPropName,
};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::auth::User;
use rustical_store::AddressbookStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, VariantNames};

pub struct PrincipalResourceService<A: AddressbookStore + ?Sized> {
    principal: String,
    addr_store: Arc<A>,
}

#[derive(Debug, Clone)]
pub struct PrincipalResource {
    principal: String,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    principal: (),
    collection: (),
}

#[derive(Deserialize, Serialize, From, TryInto)]
#[serde(rename_all = "kebab-case")]
pub enum PrincipalProp {
    // WebDAV Access Control (RFC 3744)
    #[serde(rename = "principal-URL")]
    PrincipalUrl(HrefElement),

    // CardDAV (RFC 6352)
    #[serde(rename = "CARD:addressbook-home-set")]
    AddressbookHomeSet(HrefElement),
    #[serde(rename = "CARD:principal-address")]
    PrincipalAddress(Option<HrefElement>),

    #[serde(skip_deserializing, untagged)]
    #[from]
    #[try_into]
    ExtCommonProperties(CommonPropertiesProp<PrincipalResource>),

    #[serde(untagged)]
    Invalid,
}

impl InvalidProperty for PrincipalProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(EnumString, VariantNames, Clone, From, TryInto)]
#[strum(serialize_all = "kebab-case")]
pub enum PrincipalPropName {
    #[strum(serialize = "principal-URL")]
    PrincipalUrl,
    AddressbookHomeSet,
    PrincipalAddress,
    #[from]
    #[try_into]
    #[strum(disabled)]
    ExtCommonProperties(CommonPropertiesPropName),
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
    type ResourceType = Resourcetype;

    fn list_extensions() -> Vec<BoxedExtension<Self>> {
        vec![BoxedExtension::from_ext(CommonPropertiesExtension::<
            PrincipalResource,
        >::default())]
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
            _ => panic!("we shouldn't end up here"),
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
