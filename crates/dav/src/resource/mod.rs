use crate::privileges::UserPrivilegeSet;
use crate::xml::multistatus::{PropTagWrapper, PropstatElement, PropstatWrapper};
use crate::xml::{multistatus::ResponseElement, TagList};
use crate::xml::{HrefElement, Resourcetype};
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::error::UrlGenerationError;
use actix_web::test::TestRequest;
use actix_web::{http::StatusCode, ResponseError};
pub use invalid_property::InvalidProperty;
use itertools::Itertools;
pub use resource_service::ResourceService;
use rustical_store::auth::User;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{EnumString, VariantNames};

mod invalid_property;
mod resource_service;

pub trait ResourceProp: InvalidProperty + Serialize + for<'de> Deserialize<'de> {}
impl<T: InvalidProperty + Serialize + for<'de> Deserialize<'de>> ResourceProp for T {}

pub trait ResourcePropName: FromStr + VariantNames {}
impl<T: FromStr + VariantNames> ResourcePropName for T {}

pub trait ResourceType: Serialize + for<'de> Deserialize<'de> {}
impl<T: Serialize + for<'de> Deserialize<'de>> ResourceType for T {}

#[derive(Deserialize, Serialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub enum CommonPropertiesProp {
    // WebDAV (RFC 2518)
    #[serde(skip_deserializing)]
    Resourcetype(Resourcetype),

    // WebDAV Current Principal Extension (RFC 5397)
    CurrentUserPrincipal(HrefElement),

    // WebDAV Access Control Protocol (RFC 3477)
    CurrentUserPrivilegeSet(UserPrivilegeSet),
    Owner(Option<HrefElement>),

    #[serde(other)]
    #[default]
    Invalid,
}

#[derive(Serialize)]
pub enum EitherProp<Left: ResourceProp, Right: ResourceProp> {
    #[serde(untagged)]
    Left(Left),
    #[serde(untagged)]
    Right(Right),
}

#[derive(EnumString, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum CommonPropertiesPropName {
    Resourcetype,
    CurrentUserPrincipal,
    CurrentUserPrivilegeSet,
    Owner,
}

pub trait Resource: Clone + 'static {
    type PropName: ResourcePropName;
    type Prop: ResourceProp + PartialEq;
    type Error: ResponseError + From<crate::Error>;
    type PrincipalResource: Resource;

    fn get_resourcetype() -> &'static [&'static str];

    fn list_props() -> Vec<&'static str> {
        [Self::PropName::VARIANTS, CommonPropertiesPropName::VARIANTS].concat()
    }

    fn get_internal_prop(
        &self,
        rmap: &ResourceMap,
        user: &User,
        prop: &CommonPropertiesPropName,
    ) -> Result<CommonPropertiesProp, Self::Error> {
        Ok(match prop {
            CommonPropertiesPropName::Resourcetype => {
                CommonPropertiesProp::Resourcetype(Resourcetype(Self::get_resourcetype()))
            }
            CommonPropertiesPropName::CurrentUserPrincipal => {
                CommonPropertiesProp::CurrentUserPrincipal(
                    Self::PrincipalResource::get_url(rmap, [&user.id])
                        .unwrap()
                        .into(),
                )
            }
            CommonPropertiesPropName::CurrentUserPrivilegeSet => {
                CommonPropertiesProp::CurrentUserPrivilegeSet(self.get_user_privileges(user)?)
            }
            CommonPropertiesPropName::Owner => {
                CommonPropertiesProp::Owner(self.get_owner().map(|owner| {
                    Self::PrincipalResource::get_url(rmap, [owner])
                        .unwrap()
                        .into()
                }))
            }
        })
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        user: &User,
        prop: &Self::PropName,
    ) -> Result<Self::Prop, Self::Error>;

    fn set_prop(&mut self, _prop: Self::Prop) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn remove_prop(&mut self, _prop: &Self::PropName) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn resource_name() -> &'static str;

    fn get_owner(&self) -> Option<&str> {
        None
    }

    fn get_url<U, I>(rmap: &ResourceMap, elements: U) -> Result<String, UrlGenerationError>
    where
        U: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        Ok(rmap
            .url_for(
                &TestRequest::default().to_http_request(),
                Self::resource_name(),
                elements,
            )?
            .path()
            .to_owned())
    }

    fn get_user_privileges(&self, user: &User) -> Result<UserPrivilegeSet, Self::Error>;

    fn propfind(
        &self,
        path: &str,
        mut props: Vec<&str>,
        user: &User,
        rmap: &ResourceMap,
    ) -> Result<
        ResponseElement<PropstatWrapper<EitherProp<Self::Prop, CommonPropertiesProp>>>,
        Self::Error,
    > {
        if props.contains(&"propname") {
            if props.len() != 1 {
                // propname MUST be the only queried prop per spec
                return Err(
                    Error::BadRequest("propname MUST be the only queried prop".to_owned()).into(),
                );
            }
            let props: Vec<String> = Self::list_props()
                .iter()
                .map(|&prop| prop.to_string())
                .collect();

            return Ok(ResponseElement {
                href: path.to_owned(),
                propstat: vec![PropstatWrapper::TagList(PropstatElement {
                    prop: TagList::from(props),
                    status: format!("HTTP/1.1 {}", StatusCode::OK),
                })],
                ..Default::default()
            });
        }
        if props.contains(&"allprop") {
            if props.len() != 1 {
                // allprop MUST be the only queried prop per spec
                return Err(
                    Error::BadRequest("allprop MUST be the only queried prop".to_owned()).into(),
                );
            }
            props = Self::list_props();
        }

        let mut valid_props = vec![];
        let mut internal_props = vec![];
        let mut invalid_props = vec![];
        for prop in props {
            if let Ok(valid_prop) = Self::PropName::from_str(prop) {
                valid_props.push(valid_prop);
            } else if let Ok(internal_prop) = CommonPropertiesPropName::from_str(prop) {
                internal_props.push(internal_prop);
            } else {
                invalid_props.push(prop)
            }
        }

        let internal_prop_responses: Vec<_> = internal_props
            .into_iter()
            .map(|prop| self.get_internal_prop(rmap, user, &prop))
            .collect::<Result<Vec<CommonPropertiesProp>, Self::Error>>()?
            .into_iter()
            .map(EitherProp::Right)
            .collect();

        let mut prop_responses = valid_props
            .into_iter()
            .map(|prop| self.get_prop(rmap, user, &prop))
            .map_ok(EitherProp::Left)
            .collect::<Result<Vec<_>, Self::Error>>()?;
        prop_responses.extend(internal_prop_responses);

        let mut propstats = vec![PropstatWrapper::Normal(PropstatElement {
            status: format!("HTTP/1.1 {}", StatusCode::OK),
            prop: PropTagWrapper {
                prop: prop_responses,
            },
        })];
        if !invalid_props.is_empty() {
            propstats.push(PropstatWrapper::TagList(PropstatElement {
                status: format!("HTTP/1.1 {}", StatusCode::NOT_FOUND),
                prop: invalid_props
                    .into_iter()
                    .map(|s| s.to_owned())
                    .collect_vec()
                    .into(),
            }));
        }
        Ok(ResponseElement {
            href: path.to_owned(),
            propstat: propstats,
            ..Default::default()
        })
    }
}
