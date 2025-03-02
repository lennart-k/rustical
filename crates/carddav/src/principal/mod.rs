use crate::addressbook::resource::AddressbookResource;
use crate::Error;
use async_trait::async_trait;
use educe::Educe;
use rustical_dav::extensions::{CommonPropertiesExtension, CommonPropertiesProp};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_store::auth::{AuthenticationProvider, User};
use rustical_store::AddressbookStore;
use rustical_xml::{EnumUnitVariants, EnumVariants, XmlDeserialize, XmlSerialize};
use std::sync::Arc;

#[derive(Educe)]
#[educe(Clone)]
pub struct PrincipalResourceService<A: AddressbookStore, AP: AuthenticationProvider> {
    addr_store: Arc<A>,
    auth_provider: Arc<AP>,
}

impl<A: AddressbookStore, AP: AuthenticationProvider> PrincipalResourceService<A, AP> {
    pub fn new(addr_store: Arc<A>, auth_provider: Arc<AP>) -> Self {
        Self {
            addr_store,
            auth_provider,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrincipalResource {
    principal: User,
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone)]
pub struct AddressbookHomeSet(#[xml(ty = "untagged", flatten)] Vec<HrefElement>);

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "PrincipalPropName")]
pub enum PrincipalProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(String),

    // WebDAV Access Control (RFC 3744)
    // #[xml(rename = b"principal-URL")]
    // #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    // PrincipalUrl(HrefElement),
    // CardDAV (RFC 6352)
    // #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    // AddressbookHomeSet(AddressbookHomeSet),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    PrincipalAddress(Option<HrefElement>),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, EnumUnitVariants)]
#[xml(unit_variants_ident = "PrincipalPropWrapperName", untagged)]
pub enum PrincipalPropWrapper {
    Principal(PrincipalProp),
    Common(CommonPropertiesProp),
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
        user: &User,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        // let principal_href = HrefElement::new(Self::get_principal_url(rmap, &self.principal.id));
        //
        // let home_set = AddressbookHomeSet(
        //     user.memberships()
        //         .into_iter()
        //         .map(|principal| Self::get_url(rmap, vec![principal]).unwrap())
        //         .map(HrefElement::new)
        //         .collect(),
        // );

        Ok(match prop {
            PrincipalPropWrapperName::Principal(prop) => {
                PrincipalPropWrapper::Principal(match prop {
                    PrincipalPropName::Displayname => PrincipalProp::Displayname(
                        self.principal
                            .displayname
                            .to_owned()
                            .unwrap_or(self.principal.id.to_owned()),
                    ),
                    // PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_href),
                    // PrincipalPropName::AddressbookHomeSet => {
                    //     PrincipalProp::AddressbookHomeSet(home_set)
                    // }
                    PrincipalPropName::PrincipalAddress => PrincipalProp::PrincipalAddress(None),
                })
            }

            PrincipalPropWrapperName::Common(prop) => {
                PrincipalPropWrapper::Common(CommonPropertiesExtension::get_prop(self, user, prop)?)
            }
        })
    }

    fn get_owner(&self) -> Option<&str> {
        Some(&self.principal.id)
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error> {
        Ok(UserPrivilegeSet::owner_only(
            user.is_principal(&self.principal.id),
        ))
    }
}

#[async_trait]
impl<A: AddressbookStore, AP: AuthenticationProvider> ResourceService
    for PrincipalResourceService<A, AP>
{
    type PathComponents = (String,);
    type MemberType = AddressbookResource;
    type Resource = PrincipalResource;
    type Error = Error;

    async fn get_resource(
        &self,
        (principal,): &Self::PathComponents,
    ) -> Result<Self::Resource, Self::Error> {
        let user = self
            .auth_provider
            .get_principal(principal)
            .await?
            .ok_or(crate::Error::NotFound)?;
        Ok(PrincipalResource { principal: user })
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
