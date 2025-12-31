use crate::Error;
use crate::privileges::UserPrivilege;
use crate::resource::Resource;
use crate::resource::ResourceService;
use axum::extract::{Path, State};
use axum_extra::TypedHeader;
use headers::{IfMatch, IfNoneMatch};
use http::HeaderMap;

pub async fn axum_route_delete<R: ResourceService>(
    Path(path): Path<R::PathComponents>,
    State(resource_service): State<R>,
    principal: R::Principal,
    mut if_match: Option<TypedHeader<IfMatch>>,
    mut if_none_match: Option<TypedHeader<IfNoneMatch>>,
    header_map: HeaderMap,
) -> Result<(), R::Error> {
    // https://github.com/hyperium/headers/issues/204
    if !header_map.contains_key("If-Match") {
        if_match = None;
    }
    if !header_map.contains_key("If-None-Match") {
        if_none_match = None;
    }
    let no_trash = header_map
        .get("X-No-Trashbin")
        .is_some_and(|val| matches!(val.to_str(), Ok("1")));
    route_delete(
        &path,
        &principal,
        &resource_service,
        no_trash,
        if_match.map(|hdr| hdr.0),
        if_none_match.map(|hdr| hdr.0),
    )
    .await
}

pub async fn route_delete<R: ResourceService>(
    path_components: &R::PathComponents,
    principal: &R::Principal,
    resource_service: &R,
    no_trash: bool,
    if_match: Option<IfMatch>,
    if_none_match: Option<IfNoneMatch>,
) -> Result<(), R::Error> {
    let resource = resource_service.get_resource(path_components, true).await?;

    // Kind of a bodge since we don't get unbind from the parent
    let privileges = resource.get_user_privileges(principal)?;
    if !privileges.has(&UserPrivilege::WriteProperties) {
        return Err(Error::Unauthorized.into());
    }

    if let Some(if_match) = if_match {
        if !resource.satisfies_if_match(&if_match) {
            // Precondition failed
            return Err(crate::Error::PreconditionFailed.into());
        }
    }
    if let Some(if_none_match) = if_none_match
        && resource.satisfies_if_none_match(&if_none_match)
    {
        // Precondition failed
        return Err(crate::Error::PreconditionFailed.into());
    }
    resource_service
        .delete_resource(path_components, !no_trash)
        .await?;
    Ok(())
}
