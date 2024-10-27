use crate::principal::PrincipalResource;
use crate::Error;
use actix_web::dev::ResourceMap;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

#[derive(EnumString, Debug, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum RootPropName {
    Resourcetype,
    // Defined by RFC 5397
    CurrentUserPrincipal,
}

#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    collection: (),
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum RootProp {
    // WebDAV (RFC 2518)
    Resourcetype(Resourcetype),

    // WebDAV Current Principal Extension (RFC 5397)
    CurrentUserPrincipal(HrefElement),
    #[serde(other)]
    Invalid,
}

impl InvalidProperty for RootProp {
    fn invalid_property(&self) -> bool {
        matches!(self, Self::Invalid)
    }
}

#[derive(Clone)]
pub struct RootResource {
    principal: String,
}

impl Resource for RootResource {
    type PropName = RootPropName;
    type Prop = RootProp;
    type Error = Error;

    fn get_prop(
        &self,
        rmap: &ResourceMap,
        prop: Self::PropName,
    ) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            RootPropName::Resourcetype => RootProp::Resourcetype(Resourcetype::default()),
            RootPropName::CurrentUserPrincipal => RootProp::CurrentUserPrincipal(HrefElement::new(
                PrincipalResource::get_url(rmap, vec![&self.principal]).unwrap(),
            )),
        })
    }

    #[inline]
    fn resource_name() -> &'static str {
        "caldav_root"
    }
}

pub struct RootResourceService;

#[async_trait(?Send)]
impl ResourceService for RootResourceService {
    type PathComponents = ();
    type MemberType = RootResource;
    type Resource = RootResource;
    type Error = Error;

    async fn new(
        _req: &HttpRequest,
        _path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        Ok(Self)
    }

    async fn get_resource(&self, principal: String) -> Result<Self::Resource, Self::Error> {
        Ok(RootResource { principal })
    }

    async fn save_resource(&self, _file: Self::Resource) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }
}
