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

mod axum_methods;
mod axum_service;
mod methods;
mod principal_uri;
mod resource_service;

pub use axum_methods::{AxumMethods, MethodFunction};
pub use axum_service::AxumService;
pub use principal_uri::PrincipalUri;

pub trait ResourceProp: XmlSerialize + XmlDeserialize {}
impl<T: XmlSerialize + XmlDeserialize> ResourceProp for T {}

pub trait ResourcePropName: FromStr {}
impl<T: FromStr> ResourcePropName for T {}

pub trait ResourceName {
    fn get_name(&self) -> String;
}

pub trait Resource: Clone + Send + 'static {
    type Prop: ResourceProp + PartialEq + Clone + EnumVariants + PropName + Send;
    type Error: From<crate::Error>;
    type Principal: Principal;

    fn is_collection(&self) -> bool;

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

    fn get_displayname(&self) -> Option<&str>;
    fn set_displayname(&mut self, _name: Option<String>) -> Result<(), crate::Error> {
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

    fn propfind(
        &self,
        path: &str,
        prop: &PropfindType<<Self::Prop as PropName>::Names>,
        include: Option<&PropElement<<Self::Prop as PropName>::Names>>,
        principal_uri: &impl PrincipalUri,
        principal: &Self::Principal,
    ) -> Result<ResponseElement<Self::Prop>, Self::Error> {
        // Collections have a trailing slash
        let mut path = path.to_string();
        if self.is_collection() && !path.ends_with('/') {
            path.push('/');
        }

        let (mut props, mut invalid_props): (HashSet<<Self::Prop as PropName>::Names>, Vec<_>) =
            match prop {
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

        if let Some(PropElement(valid_tags, invalid_tags)) = include {
            props.extend(valid_tags.clone());
            invalid_props.extend(invalid_tags.to_owned());
        }

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
