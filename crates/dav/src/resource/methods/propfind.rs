use crate::Error;
use crate::header::Depth;
use crate::privileges::UserPrivilege;
use crate::resource::PrincipalUri;
use crate::resource::Resource;
use crate::resource::ResourceName;
use crate::resource::ResourceService;
use crate::xml::MultistatusElement;
use crate::xml::PropfindElement;
use crate::xml::PropfindType;
use axum::extract::{Extension, OriginalUri, Path, State};
use rustical_xml::PropName;
use rustical_xml::XmlDocument;
use tracing::instrument;

type RSMultistatus<R> = MultistatusElement<
    <<R as ResourceService>::Resource as Resource>::Prop,
    <<R as ResourceService>::MemberType as Resource>::Prop,
>;

#[instrument(skip(path, resource_service, puri))]
pub(crate) async fn axum_route_propfind<R: ResourceService>(
    Path(path): Path<R::PathComponents>,
    State(resource_service): State<R>,
    depth: Depth,
    principal: R::Principal,
    uri: OriginalUri,
    Extension(puri): Extension<R::PrincipalUri>,
    body: String,
) -> Result<RSMultistatus<R>, R::Error> {
    route_propfind::<R>(
        &path,
        uri.path(),
        &body,
        &principal,
        &depth,
        &resource_service,
        &puri,
    )
    .await
}

pub(crate) async fn route_propfind<R: ResourceService>(
    path_components: &R::PathComponents,
    path: &str,
    body: &str,
    principal: &R::Principal,
    depth: &Depth,
    resource_service: &R,
    puri: &impl PrincipalUri,
) -> Result<RSMultistatus<R>, R::Error> {
    let resource = resource_service
        .get_resource(path_components, false)
        .await?;
    let privileges = resource.get_user_privileges(principal)?;
    if !privileges.has(&UserPrivilege::Read) {
        return Err(Error::Unauthorized.into());
    }

    // A request body is optional. If empty we MUST return all props
    let propfind_self = R::Resource::parse_propfind(body).map_err(Error::XmlError)?;
    let propfind_member = R::MemberType::parse_propfind(body).map_err(Error::XmlError)?;

    let mut member_responses = Vec::new();
    if depth != &Depth::Zero {
        // TODO: authorization check for member resources
        for member in resource_service.get_members(path_components).await? {
            member_responses.push(member.propfind(
                &format!("{}/{}", path.trim_end_matches('/'), member.get_name()),
                &propfind_member.prop,
                propfind_member.include.as_ref(),
                puri,
                principal,
            )?);
        }
    }

    let response = resource.propfind(
        path,
        &propfind_self.prop,
        propfind_self.include.as_ref(),
        puri,
        principal,
    )?;

    Ok(MultistatusElement {
        responses: vec![response],
        member_responses,
        ..Default::default()
    })
}
