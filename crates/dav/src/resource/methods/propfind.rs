use crate::depth_header::Depth;
use crate::privileges::UserPrivilege;
use crate::resource::Resource;
use crate::resource::ResourceService;
use crate::resource::ResourceServiceRouterState;
use crate::xml::MultistatusElement;
use crate::xml::PropElement;
use crate::xml::PropfindElement;
use crate::xml::PropfindType;
use crate::Error;
use axum::extract::OriginalUri;
use axum::extract::Path;
use axum::extract::State;
use axum::http::Uri;
use rustical_store::auth::AuthenticationProvider;
use rustical_store::auth::User;
use rustical_xml::XmlDocument;

pub(crate) async fn handle_propfind<AP: AuthenticationProvider, RS: ResourceService>(
    Path(path): Path<RS::PathComponents>,
    user: User,
    depth: Depth,
    State(ResourceServiceRouterState {
        resource_service, ..
    }): State<ResourceServiceRouterState<AP, RS>>,
    uri: Uri,
    orig_uri: OriginalUri,
    body: String,
) -> Result<
    MultistatusElement<<RS::Resource as Resource>::Prop, <RS::MemberType as Resource>::Prop>,
    RS::Error,
> {
    let prefix = orig_uri.path().trim_end_matches(uri.path());

    let resource = resource_service.get_resource(&path).await?;
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
            .map(|propname| propname.0.as_str())
            .collect(),
    };

    let members = resource_service.get_members(&path).await?;
    let mut member_responses = Vec::new();
    if depth != Depth::Zero {
        for (subpath, member) in members {
            member_responses.push(member.propfind(
                prefix,
                &format!("{}/{}", uri.path().trim_end_matches('/'), subpath),
                &props,
                &user,
            )?);
        }
    }
    let response = resource.propfind(prefix, uri.path(), &props, &user)?;

    Ok(MultistatusElement {
        responses: vec![response],
        member_responses,
        ..Default::default()
    })
}
