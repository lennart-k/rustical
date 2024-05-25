use actix_web::HttpRequest;
use anyhow::Result;
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::dav_resource::{Resource, ResourceService};
use rustical_dav::error::Error;
use rustical_dav::xml_snippets::HrefElement;
use serde::Serialize;
use strum::{EnumString, IntoStaticStr, VariantNames};

pub struct RootResource {
    prefix: String,
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
    pub prefix: String,
    pub principal: String,
    pub path: String,
}

impl Resource for RootFile {
    type PropType = RootProp;
    type PropResponse = RootPropResponse;

    fn get_prop(&self, prop: Self::PropType) -> Result<Self::PropResponse> {
        match prop {
            RootProp::Resourcetype => Ok(RootPropResponse::Resourcetype(Resourcetype::default())),
            RootProp::CurrentUserPrincipal => Ok(RootPropResponse::CurrentUserPrincipal(
                HrefElement::new(format!("{}/{}/", self.prefix, self.principal)),
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

    async fn get_members(&self, _auth_info: AuthInfo) -> Result<Vec<Self::MemberType>> {
        Ok(vec![])
    }

    async fn new(
        req: HttpRequest,
        auth_info: AuthInfo,
        _path_components: Self::PathComponents,
        prefix: String,
    ) -> Result<Self, Error> {
        Ok(Self {
            prefix,
            principal: auth_info.user_id,
            path: req.path().to_string(),
        })
    }

    async fn get_file(&self) -> Result<Self::File> {
        Ok(RootFile {
            path: self.path.to_owned(),
            principal: self.principal.to_owned(),
            prefix: self.prefix.to_owned(),
        })
    }
}
