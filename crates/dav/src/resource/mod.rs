use crate::Principal;
use crate::privileges::UserPrivilegeSet;
use crate::xml::multistatus::{PropTagWrapper, PropstatElement, PropstatWrapper};
use crate::xml::{PropElement, PropfindType, Resourcetype};
use crate::xml::{TagList, multistatus::ResponseElement};
use headers::{ETag, IfMatch, IfNoneMatch};
use http::StatusCode;
use itertools::Itertools;
use quick_xml::name::Namespace;
pub use resource_service::ResourceService;
use rustical_xml::{EnumVariants, NamespaceOwned, PropName, XmlDeserialize, XmlSerialize};
use std::collections::HashSet;
use std::str::FromStr;

#[cfg(feature = "axum")]
mod axum_methods;
#[cfg(feature = "axum")]
mod axum_service;
mod methods;
mod principal_uri;
mod resource_service;

#[cfg(feature = "axum")]
pub use axum_methods::AxumMethods;
#[cfg(feature = "axum")]
pub use axum_service::AxumService;
pub use principal_uri::PrincipalUri;

pub trait ResourceProp: XmlSerialize + XmlDeserialize {}
impl<T: XmlSerialize + XmlDeserialize> ResourceProp for T {}

pub trait ResourcePropName: FromStr {}
impl<T: FromStr> ResourcePropName for T {}

pub trait Resource: Clone + Send + 'static {
    type Prop: ResourceProp + PartialEq + Clone + EnumVariants + PropName + Send;
    type Error: From<crate::Error>;
    type Principal: Principal;

    fn get_resourcetype(&self) -> Resourcetype;

    fn list_props() -> Vec<(Option<Namespace<'static>>, &'static str)> {
        Self::Prop::variant_names()
    }

    fn get_prop(
        &self,
        principal_uri: &impl PrincipalUri,
        principal: &Self::Principal,
        prop: &<Self::Prop as PropName>::Names,
    ) -> Result<Self::Prop, Self::Error>;

    fn set_prop(&mut self, _prop: Self::Prop) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn remove_prop(&mut self, _prop: &<Self::Prop as PropName>::Names) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn get_owner(&self) -> Option<&str> {
        None
    }

    fn get_etag(&self) -> Option<String> {
        None
    }

    fn satisfies_if_match(&self, if_match: &IfMatch) -> bool {
        if let Some(etag) = self.get_etag() {
            if let Ok(etag) = ETag::from_str(&etag) {
                if_match.precondition_passes(&etag)
            } else {
                if_match.is_any()
            }
        } else {
            if_match.is_any()
        }
    }

    fn satisfies_if_none_match(&self, if_none_match: &IfNoneMatch) -> bool {
        if let Some(etag) = self.get_etag() {
            if let Ok(etag) = ETag::from_str(&etag) {
                if_none_match.precondition_passes(&etag)
            } else {
                if_none_match != &IfNoneMatch::any()
            }
        } else {
            if_none_match != &IfNoneMatch::any()
        }
    }

    fn get_user_privileges(
        &self,
        principal: &Self::Principal,
    ) -> Result<UserPrivilegeSet, Self::Error>;

    fn propfind_typed(
        &self,
        path: &str,
        prop: &PropfindType<<Self::Prop as PropName>::Names>,
        principal_uri: &impl PrincipalUri,
        principal: &Self::Principal,
    ) -> Result<ResponseElement<Self::Prop>, Self::Error> {
        // TODO: Support include element
        let (props, invalid_props): (HashSet<<Self::Prop as PropName>::Names>, Vec<_>) = match prop
        {
            PropfindType::Propname => {
                let props = Self::list_props()
                    .into_iter()
                    .map(|(ns, tag)| (ns.map(NamespaceOwned::from), tag.to_string()))
                    .collect_vec();

                return Ok(ResponseElement {
                    href: path.to_owned(),
                    propstat: vec![PropstatWrapper::TagList(PropstatElement {
                        prop: TagList::from(props),
                        status: StatusCode::OK,
                    })],
                    ..Default::default()
                });
            }
            PropfindType::Allprop => (
                Self::list_props()
                    .iter()
                    .map(|(_ns, name)| <Self::Prop as PropName>::Names::from_str(name).unwrap())
                    .collect(),
                vec![],
            ),
            PropfindType::Prop(PropElement(valid_tags, invalid_tags)) => (
                valid_tags.iter().cloned().collect(),
                invalid_tags.to_owned(),
            ),
        };

        let prop_responses = props
            .into_iter()
            .map(|prop| self.get_prop(principal_uri, principal, &prop))
            .collect::<Result<Vec<_>, Self::Error>>()?;

        let mut propstats = vec![PropstatWrapper::Normal(PropstatElement {
            status: StatusCode::OK,
            prop: PropTagWrapper(prop_responses),
        })];
        if !invalid_props.is_empty() {
            propstats.push(PropstatWrapper::TagList(PropstatElement {
                status: StatusCode::NOT_FOUND,
                prop: invalid_props.into(),
            }));
        }
        Ok(ResponseElement {
            href: path.to_owned(),
            propstat: propstats,
            ..Default::default()
        })
    }
}
