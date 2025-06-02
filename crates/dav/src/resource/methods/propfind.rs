use crate::Error;
use crate::header::Depth;
use crate::privileges::UserPrivilege;
use crate::resource::PrincipalUri;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::xml::MultistatusElement;
use crate::xml::PropElement;
use crate::xml::PropfindElement;
use crate::xml::PropfindType;
use rustical_xml::XmlDocument;
use tracing::instrument;

#[cfg(feature = "actix")]
#[instrument(parent = root_span.id(), skip(path, req, root_span, resource_service, puri))]
#[allow(clippy::type_complexity)]
pub(crate) async fn actix_route_propfind<R: ResourceService>(
    path: ::actix_web::web::Path<R::PathComponents>,
    body: String,
    req: ::actix_web::HttpRequest,
    user: R::Principal,
    depth: Depth,
    root_span: tracing_actix_web::RootSpan,
    resource_service: ::actix_web::web::Data<R>,
    puri: ::actix_web::web::Data<R::PrincipalUri>,
) -> Result<
    MultistatusElement<<R::Resource as Resource>::Prop, <R::MemberType as Resource>::Prop>,
    R::Error,
> {
    route_propfind(
        &path.into_inner(),
        req.path(),
        body,
        user,
        depth,
        resource_service.as_ref(),
        puri.as_ref(),
    )
    .await
}

pub(crate) async fn route_propfind<R: ResourceService>(
    path_components: &R::PathComponents,
    path: &str,
    body: String,
    user: R::Principal,
    depth: Depth,
    resource_service: &R,
    puri: &impl PrincipalUri,
) -> Result<
    MultistatusElement<<R::Resource as Resource>::Prop, <R::MemberType as Resource>::Prop>,
    R::Error,
> {
    let resource = resource_service.get_resource(path_components).await?;
    let privileges = resource.get_user_privileges(&user)?;
    if !privileges.has(&UserPrivilege::Read) {
        return Err(Error::Unauthorized.into());
    }

    // A request body is optional. If empty we MUST return all props
    let propfind: PropfindElement = if !body.is_empty() {
        PropfindElement::parse_str(&body).map_err(Error::XmlError)?
    } else {
        PropfindElement {
            prop: PropfindType::Allprop,
        }
    };

    // TODO: respect namespaces?
    let props = match &propfind.prop {
        PropfindType::Allprop => vec!["allprop"],
        PropfindType::Propname => vec!["propname"],
        PropfindType::Prop(PropElement(prop_tags)) => prop_tags
            .iter()
            .map(|propname| propname.name.as_str())
            .collect(),
    };

    let mut member_responses = Vec::new();
    if depth != Depth::Zero {
        for (subpath, member) in resource_service.get_members(path_components).await? {
            member_responses.push(member.propfind(
                &format!("{}/{}", path.trim_end_matches('/'), subpath),
                &props,
                puri,
                &user,
            )?);
        }
    }

    let response = resource.propfind(path, &props, puri, &user)?;

    Ok(MultistatusElement {
        responses: vec![response],
        member_responses,
        ..Default::default()
    })
}
