use crate::Error;
use actix_web::HttpRequest;
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::resource::{InvalidProperty, Resource, ResourceService};
use rustical_dav::xml::HrefElement;
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

pub struct RootResource {
    principal: String,
    path: String,
}

#[derive(EnumString, Debug, VariantNames, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum RootPropName {
    Resourcetype,
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
pub struct RootFile {
    pub principal: String,
    pub path: String,
}

impl Resource for RootFile {
    type PropName = RootPropName;
    type Prop = RootProp;
    type Error = Error;

    fn get_prop(&self, prefix: &str, prop: Self::PropName) -> Result<Self::Prop, Self::Error> {
        match prop {
            RootPropName::Resourcetype => Ok(RootProp::Resourcetype(Resourcetype::default())),
            RootPropName::CurrentUserPrincipal => Ok(RootProp::CurrentUserPrincipal(
                HrefElement::new(format!("{}/{}/", prefix, self.principal)),
            )),
        }
    }

    fn get_path(&self) -> &str {
        &self.path
    }
}

#[async_trait(?Send)]
impl ResourceService for RootResource {
    type PathComponents = ();
    type MemberType = RootFile;
    type File = RootFile;
    type Error = Error;

    async fn get_members(
        &self,
        _auth_info: AuthInfo,
    ) -> Result<Vec<Self::MemberType>, Self::Error> {
        Ok(vec![])
    }

    async fn new(
        req: HttpRequest,
        auth_info: AuthInfo,
        _path_components: Self::PathComponents,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            principal: auth_info.user_id,
            path: req.path().to_string(),
        })
    }

    async fn get_file(&self) -> Result<Self::File, Self::Error> {
        Ok(RootFile {
            path: self.path.to_owned(),
            principal: self.principal.to_owned(),
        })
    }

    async fn save_file(&self, _file: Self::File) -> Result<(), Self::Error> {
        Err(Error::NotImplemented)
    }
}
