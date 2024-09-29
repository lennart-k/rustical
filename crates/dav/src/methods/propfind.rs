use crate::depth_extractor::Depth;
use crate::resource::HandlePropfind;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::xml::multistatus::PropstatWrapper;
use crate::xml::MultistatusElement;
use crate::xml::TagList;
use crate::Error;
use actix_web::web::{Data, Path};
use actix_web::HttpRequest;
use derive_more::derive::Deref;
use log::debug;
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use serde::Deserialize;

// This is not the final place for this struct
#[derive(Deref)]
pub struct ServicePrefix(pub String);

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PropElement {
    #[serde(flatten)]
    pub prop: TagList,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PropfindType {
    Propname,
    Allprop,
    Prop(PropElement),
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
struct PropfindElement {
    #[serde(rename = "$value")]
    prop: PropfindType,
}

pub async fn route_propfind<A: CheckAuthentication, R: ResourceService>(
    path_components: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    prefix: Data<ServicePrefix>,
    auth: AuthInfoExtractor<A>,
    depth: Depth,
) -> Result<
    MultistatusElement<
        PropstatWrapper<<R::Resource as Resource>::Prop>,
        PropstatWrapper<<R::MemberType as Resource>::Prop>,
    >,
    R::Error,
> {
    debug!("{body}");
    let prefix = prefix.into_inner();
    let path = req.path().to_owned();

    let resource_service = R::new(&req, path_components.into_inner()).await?;

    // A request body is optional. If empty we MUST return all props
    let propfind: PropfindElement = if !body.is_empty() {
        quick_xml::de::from_str(&body).map_err(Error::XmlDecodeError)?
    } else {
        PropfindElement {
            prop: PropfindType::Allprop,
        }
    };

    let props = match propfind.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            // TODO: Implement
            return Err(Error::InternalError.into());
        }
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags.into(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut member_responses = Vec::new();
    if depth != Depth::Zero {
        for (path, member) in resource_service.get_members().await? {
            member_responses.push(member.propfind(&prefix, path, props.clone()).await?);
        }
    }

    let resource = resource_service.get_resource(auth.inner.user_id).await?;
    let response = resource.propfind(&prefix, path, props).await?;

    Ok(MultistatusElement {
        responses: vec![response],
        member_responses,
        ..Default::default()
    })
}
