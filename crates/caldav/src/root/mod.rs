use crate::Error;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

pub struct RootResourceService {
    principal: String,
}

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
    Resourcetype(Resourcetype),
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
    pub principal: String,
}

impl Resource for RootResource {
    type PropName = RootPropName;
    type Prop = RootProp;
    type Error = Error;

    fn get_prop(&self, prefix: &str, prop: Self::PropName) -> Result<Self::Prop, Self::Error> {
        Ok(match prop {
            RootPropName::Resourcetype => RootProp::Resourcetype(Resourcetype::default()),
            RootPropName::CurrentUserPrincipal => RootProp::CurrentUserPrincipal(HrefElement::new(
                format!("{}/user/{}/", prefix, self.principal),
            )),
        })
    }
}

#[async_trait(?Send)]
impl ResourceService for RootResourceService {
    type PathComponents = ();
    type MemberType = RootResource;
    type File = RootResource;
    type Error = Error;

    async fn new(
        _req: &HttpRequest,
        auth_info: &AuthInfo,
        _path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            principal: auth_info.user_id.to_owned(),
        })
    }

    async fn get_file(&self) -> Result<Self::File, Self::Error> {
        Ok(RootResource {
            principal: self.principal.to_owned(),
        })
    }

    async fn save_file(&self, _file: Self::File) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }
}
