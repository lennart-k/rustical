use crate::resource::{Resource, ResourceProp, ResourcePropName};
use actix_web::dev::ResourceMap;
use derive_more::derive::Deref;
use rustical_store::auth::User;
use std::str::FromStr;
use strum::VariantNames;

pub trait ResourceExtension<R: Resource>: Clone {
    type PropName: ResourcePropName;
    type Prop: ResourceProp;
    type Error: Into<R::Error> + From<crate::Error>;

    fn list_props() -> &'static [&'static str] {
        Self::PropName::VARIANTS
    }

    fn get_prop(
        &self,
        resource: &R,
        rmap: &ResourceMap,
        user: &User,
        prop: Self::PropName,
    ) -> Result<Self::Prop, Self::Error>;

    fn remove_prop(&mut self, _prop: &Self::PropName) -> Result<(), Self::Error> {
        Err(crate::Error::PropReadOnly.into())
    }
}

pub struct ResourceExtensionWrapper;

pub trait BoxableExtension<R: Resource> {
    fn get_prop(
        &self,
        resource: &R,
        rmap: &ResourceMap,
        user: &User,
        prop: &str,
    ) -> Result<Option<R::Prop>, R::Error>;

    fn propfind<'a>(
        &self,
        resource: &R,
        props: Vec<&'a str>,
        user: &User,
        rmap: &ResourceMap,
    ) -> Result<(Vec<&'a str>, Vec<R::Prop>), R::Error>;

    fn list_props(&self) -> &'static [&'static str];
}

impl<R: Resource, RE: ResourceExtension<R, Prop: Into<R::Prop>, Error: Into<R::Error>>>
    BoxableExtension<R> for RE
{
    fn get_prop(
        &self,
        resource: &R,
        rmap: &ResourceMap,
        user: &User,
        prop: &str,
        // prop: <R as Resource>::PropName,
    ) -> Result<Option<R::Prop>, R::Error> {
        let prop: RE::PropName = if let Ok(prop) = prop.parse() {
            prop
        } else {
            return Ok(None);
        };

        let prop = ResourceExtension::<R>::get_prop(self, resource, rmap, user, prop)
            .map_err(RE::Error::into)?;
        Ok(Some(prop.into()))
    }

    fn propfind<'a>(
        &self,
        resource: &R,
        props: Vec<&'a str>,
        user: &User,
        rmap: &ResourceMap,
    ) -> Result<(Vec<&'a str>, Vec<R::Prop>), R::Error> {
        let (valid_props, invalid_props): (Vec<Option<RE::PropName>>, Vec<Option<&str>>) = props
            .into_iter()
            .map(|prop| {
                if let Ok(valid_prop) = RE::PropName::from_str(prop) {
                    (Some(valid_prop), None)
                } else {
                    (None, Some(prop))
                }
            })
            .unzip();
        let valid_props: Vec<RE::PropName> = valid_props.into_iter().flatten().collect();
        let invalid_props: Vec<&str> = invalid_props.into_iter().flatten().collect();

        let prop_responses = valid_props
            .into_iter()
            .map(|prop| self.get_prop(resource, rmap, user, prop))
            .collect::<Result<Vec<_>, RE::Error>>()
            .map_err(RE::Error::into)?
            .into_iter()
            .map(|prop| prop.into())
            .collect::<Vec<_>>();

        Ok((invalid_props, prop_responses))
    }

    fn list_props(&self) -> &'static [&'static str] {
        Self::list_props()
    }
}

#[derive(Deref)]
pub struct BoxedExtension<R>(Box<dyn BoxableExtension<R>>);

impl<R: Resource> BoxedExtension<R> {
    pub fn from_ext<RE: ResourceExtension<R, Prop: Into<R::Prop>> + 'static>(ext: RE) -> Self {
        let boxed_ext: Box<dyn BoxableExtension<R>> = Box::new(ext);
        BoxedExtension(boxed_ext)
    }
}
