use crate::resource::{Resource, ResourceProp, ResourcePropName};
use actix_web::dev::ResourceMap;
use rustical_store::auth::User;
use std::str::FromStr;
use strum::VariantNames;

pub trait ResourceExtension<R: Resource>: Clone {
    type PropName: ResourcePropName;
    type Prop: ResourceProp + Into<R::Prop>;
    type Error: Into<R::Error> + From<crate::Error>;

    fn list_props(&self) -> &'static [&'static str] {
        Self::PropName::VARIANTS
    }

    fn get_prop(
        &self,
        resource: &R,
        rmap: &ResourceMap,
        user: &User,
        prop: Self::PropName,
    ) -> Result<Self::Prop, Self::Error>;

    fn propfind<'a>(
        &self,
        resource: &R,
        props: Vec<&'a str>,
        user: &User,
        rmap: &ResourceMap,
    ) -> Result<(Vec<&'a str>, Vec<R::Prop>), R::Error> {
        let (valid_props, invalid_props): (Vec<Option<Self::PropName>>, Vec<Option<&str>>) = props
            .into_iter()
            .map(|prop| {
                if let Ok(valid_prop) = Self::PropName::from_str(prop) {
                    (Some(valid_prop), None)
                } else {
                    (None, Some(prop))
                }
            })
            .unzip();
        let valid_props: Vec<Self::PropName> = valid_props.into_iter().flatten().collect();
        let invalid_props: Vec<&str> = invalid_props.into_iter().flatten().collect();

        let prop_responses = valid_props
            .into_iter()
            .map(|prop| <Self as ResourceExtension<_>>::get_prop(self, resource, rmap, user, prop))
            .collect::<Result<Vec<_>, Self::Error>>()
            .map_err(Self::Error::into)?
            .into_iter()
            .map(|prop| prop.into())
            .collect::<Vec<_>>();

        Ok((invalid_props, prop_responses))
    }
}
