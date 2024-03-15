use actix_web::HttpRequest;
use anyhow::Result;
use async_trait::async_trait;
use rustical_auth::AuthInfo;
use rustical_dav::{resource::Resource, xml_snippets::HrefElement};
use serde::Serialize;
use strum::{EnumProperty, EnumString, IntoStaticStr, VariantNames};

pub struct RootResource {
    prefix: String,
    principal: String,
    path: String,
}

#[derive(EnumString, Debug, VariantNames, EnumProperty, IntoStaticStr, Clone)]
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

#[async_trait(?Send)]
impl Resource for RootResource {
    type UriComponents = ();
    type MemberType = Self;
    type PropType = RootProp;
    type PropResponse = RootPropResponse;

    fn get_path(&self) -> &str {
        &self.path
    }

    async fn get_members(&self) -> Result<Vec<Self::MemberType>> {
        Ok(vec![])
    }

    async fn acquire_from_request(
        req: HttpRequest,
        auth_info: AuthInfo,
        _uri_components: Self::UriComponents,
        prefix: String,
    ) -> Result<Self> {
        Ok(Self {
            prefix,
            principal: auth_info.user_id,
            path: req.path().to_string(),
        })
    }

    fn get_prop(&self, prop: Self::PropType) -> Result<Self::PropResponse> {
        match prop {
            RootProp::Resourcetype => Ok(RootPropResponse::Resourcetype(Resourcetype::default())),
            RootProp::CurrentUserPrincipal => Ok(RootPropResponse::CurrentUserPrincipal(
                HrefElement::new(format!("{}/{}/", self.prefix, self.principal)),
            )),
        }
    }
}
