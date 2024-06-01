use crate::Error;
use actix_web::HttpRequest;
use anyhow::Result;
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_dav::xml_snippets::HrefElement;
use serde::Serialize;
use strum::{EnumString, IntoStaticStr, VariantNames};

pub struct RootResource {
    principal: String,
    path: String,
}

#[derive(EnumString, Debug, VariantNames, IntoStaticStr, Clone)]
#[strum(serialize_all = "kebab-case")]
pub enum RootProp {
    Resourcetype,
    CurrentUserPrincipal,
}

#[derive(Serialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    collection: (),
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RootPropResponse {
    Resourcetype(Resourcetype),
    CurrentUserPrincipal(HrefElement),
}

pub struct RootFile {
    pub principal: String,
    pub path: String,
}

impl Resource for RootFile {
    type PropType = RootProp;
    type PropResponse = RootPropResponse;
    type Error = Error;

    fn get_prop(
        &self,
        prefix: &str,
        prop: Self::PropType,
    ) -> Result<Self::PropResponse, Self::Error> {
        match prop {
            RootProp::Resourcetype => Ok(RootPropResponse::Resourcetype(Resourcetype::default())),
            RootProp::CurrentUserPrincipal => Ok(RootPropResponse::CurrentUserPrincipal(
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
}
