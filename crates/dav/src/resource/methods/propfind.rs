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
    let resource = resource_service.get_resource(path_components).await?;
    let privileges = resource.get_user_privileges(principal)?;
    if !privileges.has(&UserPrivilege::Read) {
        return Err(Error::Unauthorized.into());
    }

    // A request body is optional. If empty we MUST return all props
    let propfind_self: PropfindElement<<<R::Resource as Resource>::Prop as PropName>::Names> =
        if !body.is_empty() {
            PropfindElement::parse_str(body).map_err(Error::XmlError)?
        } else {
            PropfindElement {
                prop: PropfindType::Allprop,
            }
        };
    let propfind_member: PropfindElement<<<R::MemberType as Resource>::Prop as PropName>::Names> =
        if !body.is_empty() {
            PropfindElement::parse_str(body).map_err(Error::XmlError)?
        } else {
            PropfindElement {
                prop: PropfindType::Allprop,
            }
        };

    let mut member_responses = Vec::new();
    if depth != &Depth::Zero {
        for member in resource_service.get_members(path_components).await? {
            // Collections should have a trailing slash
            let mut name = member.get_name();
            if R::IS_COLLECTION {
                name.push('/')
            }
            member_responses.push(member.propfind_typed(
                &format!("{}/{}", path.trim_end_matches('/'), name),
                &propfind_member.prop,
                puri,
                principal,
            )?);
        }
    }

    let response = resource.propfind_typed(path, &propfind_self.prop, puri, principal)?;

    Ok(MultistatusElement {
        responses: vec![response],
        member_responses,
        ..Default::default()
    })
}
