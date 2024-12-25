use crate::depth_header::Depth;
use crate::privileges::UserPrivilege;
use crate::resource::CommonPropertiesProp;
use crate::resource::EitherProp;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::xml::MultistatusElement;
use crate::xml::PropElement;
use crate::xml::PropfindElement;
use crate::xml::PropfindType;
use crate::Error;
use actix_web::web::Path;
use actix_web::HttpRequest;
use rustical_store::auth::User;
use rustical_xml::de::XmlDocument;
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[instrument(parent = root_span.id(), skip(path_components, req, root_span))]
#[allow(clippy::type_complexity)]
pub(crate) async fn route_propfind<R: ResourceService>(
    path_components: Path<R::PathComponents>,
    body: String,
    req: HttpRequest,
    user: User,
    depth: Depth,
    root_span: RootSpan,
) -> Result<
    MultistatusElement<
        EitherProp<<R::Resource as Resource>::Prop, CommonPropertiesProp>,
        EitherProp<<R::MemberType as Resource>::Prop, CommonPropertiesProp>,
    >,
    R::Error,
> {
    let resource_service = R::new(&req, path_components.into_inner()).await?;

    let resource = resource_service.get_resource().await?;
    let privileges = resource.get_user_privileges(&user)?;
    if !privileges.has(&UserPrivilege::Read) {
        return Err(Error::Unauthorized.into());
    }

    // A request body is optional. If empty we MUST return all props
    let propfind: PropfindElement = if !body.is_empty() {
        PropfindElement::parse_str(&body).map_err(Error::XmlDeserializationError)?
    } else {
        PropfindElement {
            prop: PropfindType::Allprop,
        }
    };

    let props = match propfind.prop {
        PropfindType::Allprop => vec!["allprop".to_owned()],
        PropfindType::Propname => vec!["propname".to_owned()],
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags
            .into_iter()
            .map(|propname| propname.name)
            .collect(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let mut member_responses = Vec::new();
    if depth != Depth::Zero {
        for (path, member) in resource_service.get_members(req.resource_map()).await? {
            member_responses.push(member.propfind(
                &path,
                props.clone(),
                &user,
                req.resource_map(),
            )?);
        }
    }

    let response = resource.propfind(req.path(), props, &user, req.resource_map())?;

    Ok(MultistatusElement {
        responses: vec![response],
        member_responses,
        ..Default::default()
    })
}
