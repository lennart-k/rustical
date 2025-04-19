use crate::privileges::UserPrivilegeSet;
use crate::xml::Resourcetype;
use crate::xml::multistatus::{PropTagWrapper, PropstatElement, PropstatWrapper};
use crate::xml::{TagList, multistatus::ResponseElement};
use crate::{Error, Principal};
use actix_web::dev::ResourceMap;
use actix_web::http::header::{EntityTag, IfMatch, IfNoneMatch};
use actix_web::{ResponseError, http::StatusCode};
use itertools::Itertools;
use quick_xml::name::Namespace;
pub use resource_service::ResourceService;
use rustical_xml::{EnumUnitVariants, EnumVariants, XmlDeserialize, XmlSerialize};
use std::str::FromStr;

mod methods;
mod resource_service;

pub use resource_service::*;

pub trait ResourceProp: XmlSerialize + XmlDeserialize {}
impl<T: XmlSerialize + XmlDeserialize> ResourceProp for T {}

pub trait ResourcePropName: FromStr {}
impl<T: FromStr> ResourcePropName for T {}

pub trait Resource: Clone + 'static {
    type Prop: ResourceProp + PartialEq + Clone + EnumVariants + EnumUnitVariants;
    type Error: ResponseError + From<crate::Error>;
    type Principal: Principal;

    fn get_resourcetype(&self) -> Resourcetype;

    fn list_props() -> Vec<(Option<Namespace<'static>>, &'static str)> {
        Self::Prop::variant_names()
    }

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        principal: &Self::Principal,
        prop: &<Self::Prop as EnumUnitVariants>::UnitVariants,
    ) -> Result<Self::Prop, Self::Error>;

    fn set_prop(&mut self, _prop: Self::Prop) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn remove_prop(
        &mut self,
        _prop: &<Self::Prop as EnumUnitVariants>::UnitVariants,
    ) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn get_owner(&self) -> Option<&str> {
        None
    }

    fn get_etag(&self) -> Option<String> {
        None
    }

    fn satisfies_if_match(&self, if_match: &IfMatch) -> bool {
        match if_match {
            IfMatch::Any => true,
            // This is not nice but if the header doesn't exist, actix just gives us an empty
            // IfMatch::Items header
            IfMatch::Items(items) if items.is_empty() => true,
            IfMatch::Items(items) => {
                if let Some(etag) = self.get_etag() {
                    let etag = EntityTag::new_strong(etag.to_owned());
                    return items.iter().any(|item| item.strong_eq(&etag));
                }
                false
            }
        }
    }

    fn satisfies_if_none_match(&self, if_none_match: &IfNoneMatch) -> bool {
        match if_none_match {
            IfNoneMatch::Any => false,
            // This is not nice but if the header doesn't exist, actix just gives us an empty
            // IfNoneMatch::Items header
            IfNoneMatch::Items(items) if items.is_empty() => false,
            IfNoneMatch::Items(items) => {
                if let Some(etag) = self.get_etag() {
                    let etag = EntityTag::new_strong(etag.to_owned());
                    return items.iter().all(|item| item.strong_ne(&etag));
                }
                true
            }
        }
    }

    fn get_user_privileges(
        &self,
        principal: &Self::Principal,
    ) -> Result<UserPrivilegeSet, Self::Error>;

    fn propfind(
        &self,
        path: &str,
        props: &[&str],
        principal: &Self::Principal,
        rmap: &ResourceMap,
    ) -> Result<ResponseElement<Self::Prop>, Self::Error> {
        let mut props = props.to_vec();

        if props.contains(&"propname") {
            if props.len() != 1 {
                // propname MUST be the only queried prop per spec
                return Err(
                    Error::BadRequest("propname MUST be the only queried prop".to_owned()).into(),
                );
            }

            let props = Self::list_props()
                .into_iter()
                .map(|(ns, tag)| (ns.to_owned(), tag.to_string()))
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

        if props.contains(&"allprop") {
            if props.len() != 1 {
                // allprop MUST be the only queried prop per spec
                return Err(
                    Error::BadRequest("allprop MUST be the only queried prop".to_owned()).into(),
                );
            }
            props = Self::list_props()
                .into_iter()
                .map(|(_ns, tag)| tag)
                .collect();
        }

        let mut valid_props = vec![];
        let mut invalid_props = vec![];
        for prop in props {
            if let Ok(valid_prop) = <Self::Prop as EnumUnitVariants>::UnitVariants::from_str(prop) {
                valid_props.push(valid_prop);
            } else {
                invalid_props.push(prop.to_string())
            }
        }

        let prop_responses = valid_props
            .into_iter()
            .map(|prop| self.get_prop(rmap, principal, &prop))
            .collect::<Result<Vec<_>, Self::Error>>()?;

        let mut propstats = vec![PropstatWrapper::Normal(PropstatElement {
            status: StatusCode::OK,
            prop: PropTagWrapper(prop_responses),
        })];
        if !invalid_props.is_empty() {
            propstats.push(PropstatWrapper::TagList(PropstatElement {
                status: StatusCode::NOT_FOUND,
                prop: invalid_props
                    .into_iter()
                    .map(|tag| (None, tag))
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
