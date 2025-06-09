use crate::addressbook::resource::{AddressbookResource, AddressbookResourceService};
use crate::{CardDavPrincipalUri, Error};
use async_trait::async_trait;
use axum::Router;
use rustical_dav::extensions::{CommonPropertiesExtension, CommonPropertiesProp};
use rustical_dav::privileges::UserPrivilegeSet;
use rustical_dav::resource::{AxumMethods, PrincipalUri, Resource, ResourceService};
use rustical_dav::xml::{HrefElement, Resourcetype, ResourcetypeInner};
use rustical_store::auth::{AuthenticationProvider, User};
use rustical_store::{AddressbookStore, SubscriptionStore};
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};
use std::sync::Arc;

pub struct PrincipalResourceService<
    A: AddressbookStore,
    AP: AuthenticationProvider,
    S: SubscriptionStore,
> {
    addr_store: Arc<A>,
    auth_provider: Arc<AP>,
    sub_store: Arc<S>,
}

impl<A: AddressbookStore, AP: AuthenticationProvider, S: SubscriptionStore> Clone
    for PrincipalResourceService<A, AP, S>
{
    fn clone(&self) -> Self {
        Self {
            addr_store: self.addr_store.clone(),
            auth_provider: self.auth_provider.clone(),
            sub_store: self.sub_store.clone(),
        }
    }
}

impl<A: AddressbookStore, AP: AuthenticationProvider, S: SubscriptionStore>
    PrincipalResourceService<A, AP, S>
{
    pub fn new(addr_store: Arc<A>, auth_provider: Arc<AP>, sub_store: Arc<S>) -> Self {
        Self {
            addr_store,
            auth_provider,
            sub_store,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrincipalResource {
    principal: User,
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone)]
pub struct AddressbookHomeSet(#[xml(ty = "untagged", flatten)] Vec<HrefElement>);

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
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
    AddressbookHomeSet(AddressbookHomeSet),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    PrincipalAddress(Option<HrefElement>),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "PrincipalPropWrapperName", untagged)]
pub enum PrincipalPropWrapper {
    Principal(PrincipalProp),
    Common(CommonPropertiesProp),
}

impl Resource for PrincipalResource {
    type Prop = PrincipalPropWrapper;
    type Error = Error;
    type Principal = User;

    fn get_resourcetype(&self) -> Resourcetype {
        Resourcetype(&[
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "collection"),
            ResourcetypeInner(Some(rustical_dav::namespace::NS_DAV), "principal"),
        ])
    }

    fn get_prop(
        &self,
        puri: &impl PrincipalUri,
        user: &User,
        prop: &PrincipalPropWrapperName,
    ) -> Result<Self::Prop, Self::Error> {
        let principal_href = HrefElement::new(puri.principal_uri(&user.id));

        let home_set = AddressbookHomeSet(
            user.memberships()
                .into_iter()
                .map(|principal| puri.principal_uri(principal))
                .map(HrefElement::new)
                .collect(),
        );

        Ok(match prop {
            PrincipalPropWrapperName::Principal(prop) => {
                PrincipalPropWrapper::Principal(match prop {
                    PrincipalPropName::Displayname => PrincipalProp::Displayname(
                        self.principal
                            .displayname
                            .to_owned()
                            .unwrap_or(self.principal.id.to_owned()),
                    ),
                    PrincipalPropName::PrincipalUrl => PrincipalProp::PrincipalUrl(principal_href),
                    PrincipalPropName::AddressbookHomeSet => {
                        PrincipalProp::AddressbookHomeSet(home_set)
                    }
                    PrincipalPropName::PrincipalAddress => PrincipalProp::PrincipalAddress(None),
                })
            }

            PrincipalPropWrapperName::Common(prop) => PrincipalPropWrapper::Common(
                CommonPropertiesExtension::get_prop(self, puri, user, prop)?,
            ),
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
impl<A: AddressbookStore, AP: AuthenticationProvider, S: SubscriptionStore> ResourceService
    for PrincipalResourceService<A, AP, S>
{
    type PathComponents = (String,);
    type MemberType = AddressbookResource;
    type Resource = PrincipalResource;
    type Error = Error;
    type Principal = User;
    type PrincipalUri = CardDavPrincipalUri;

    const DAV_HEADER: &str = "1, 3, access-control, addressbook";

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

    fn axum_router<State: Send + Sync + Clone + 'static>(self) -> Router<State> {
        Router::new()
            .nest(
                "/{addressbook_id}",
                AddressbookResourceService::new(self.addr_store.clone(), self.sub_store.clone())
                    .axum_router(),
            )
            .route_service("/", self.axum_service())
    }
}

impl<A: AddressbookStore, AP: AuthenticationProvider, S: SubscriptionStore> AxumMethods
    for PrincipalResourceService<A, AP, S>
{
}
