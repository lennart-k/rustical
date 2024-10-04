use crate::depth_extractor::Depth;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::xml::multistatus::PropstatWrapper;
use crate::xml::MultistatusElement;
use crate::xml::TagList;
use crate::Error;
use actix_web::web::Path;
use actix_web::HttpRequest;
use rustical_store::auth::User;
use serde::Deserialize;
use tracing::instrument;
use tracing_actix_web::RootSpan;

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

#[instrument(parent = root_span.id(), skip(path_components, req, root_span))]
pub async fn route_propfind<R: ResourceService>(
    path_components: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    user: User,
    depth: Depth,
    root_span: RootSpan,
) -> Result<
    MultistatusElement<
        PropstatWrapper<<R::Resource as Resource>::Prop>,
        PropstatWrapper<<R::MemberType as Resource>::Prop>,
    >,
    R::Error,
> {
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
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags.into_inner(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut member_responses = Vec::new();
    if depth != Depth::Zero {
        for (path, member) in resource_service.get_members(req.resource_map()).await? {
            member_responses.push(
                member
                    .propfind(&path, props.clone(), req.resource_map())
                    .await?,
            );
        }
    }

    let resource = resource_service.get_resource(user.id).await?;
    let response = resource
        .propfind(req.path(), props, req.resource_map())
        .await?;

    Ok(MultistatusElement {
        responses: vec![response],
        member_responses,
        ..Default::default()
    })
}
