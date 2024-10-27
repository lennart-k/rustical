use crate::addressbook::resource::AddressbookResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::web::Data;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use rustical_store::AddressbookStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use strum::{EnumString, VariantNames};

pub struct PrincipalResourceService<A: AddressbookStore + ?Sized> {
    principal: String,
    addr_store: Arc<A>,
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
    // WebDAV (RFC 2518)
    Resourcetype(Resourcetype),

    // WebDAV Access Control (RFC 3744)
    #[serde(rename = "principal-URL")]
    PrincipalUrl(HrefElement),

    // WebDAV Current Principal Extension (RFC 5397)
    CurrentUserPrincipal(HrefElement),

    // CardDAV (RFC 6352)
    #[serde(rename = "CARD:addressbook-home-set")]
    AddressbookHomeSet(HrefElement),
    #[serde(rename = "CARD:principal-address")]
    PrincipalAddress(Option<HrefElement>),
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
    AddressbookHomeSet,
    PrincipalAddress,
}

impl Resource for PrincipalResource {
    type PropName = PrincipalPropName;
    type Prop = PrincipalProp;
    type Error = Error;

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        prop: Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(Self::get_url(rmap, vec![&self.principal]).unwrap());

        Ok(match prop {
            PrincipalPropName::Resourcetype => PrincipalProp::Resourcetype(Resourcetype::default()),
            PrincipalPropName::CurrentUserPrincipal => {
                PrincipalProp::CurrentUserPrincipal(principal_href)
            }
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

    async fn get_resource(&self, principal: String) -> Result<Self::Resource, Self::Error> {
        if self.principal != principal {
            return Err(Error::Unauthorized);
        }
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

    async fn save_resource(&self, _file: Self::Resource) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }
}
