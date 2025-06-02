use crate::Error;
use crate::privileges::UserPrivilege;
use crate::resource::Resource;
use crate::resource::ResourceService;
use headers::Header;
use headers::{HeaderValue, IfMatch, IfNoneMatch};
use itertools::Itertools;
use tracing::instrument;

#[cfg(feature = "actix")]
#[instrument(parent = root_span.id(), skip(path, req, root_span, resource_service))]
pub async fn actix_route_delete<R: ResourceService>(
    path: actix_web::web::Path<R::PathComponents>,
    req: actix_web::HttpRequest,
    principal: R::Principal,
    resource_service: actix_web::web::Data<R>,
    root_span: tracing_actix_web::RootSpan,
) -> Result<actix_web::HttpResponse, R::Error> {
    let no_trash = req
        .headers()
        .get("X-No-Trashbin")
        .map(|val| matches!(val.to_str(), Ok("1")))
        .unwrap_or(false);

    // This weird conversion stuff is because we want to use the headers library (to be
    // framework-agnostic in the future) which uses http==1.0,
    // while actix-web still uses http==0.2
    let if_match = req
        .headers()
        .get_all(http::header::IF_MATCH)
        .map(|val_02| HeaderValue::from_bytes(val_02.as_bytes()).unwrap())
        .collect_vec();
    let if_none_match = req
        .headers()
        .get_all(http::header::IF_NONE_MATCH)
        .map(|val_02| HeaderValue::from_bytes(val_02.as_bytes()).unwrap())
        .collect_vec();

    let if_match = if if_match.is_empty() {
        None
    } else {
        Some(IfMatch::decode(&mut if_match.iter()).unwrap())
    };
    let if_none_match = if if_none_match.is_empty() {
        None
    } else {
        Some(IfNoneMatch::decode(&mut if_none_match.iter()).unwrap())
    };

    route_delete(
        &path.into_inner(),
        principal,
        resource_service.as_ref(),
        no_trash,
        if_match,
        if_none_match,
    )
    .await?;

    Ok(actix_web::HttpResponse::Ok().body(""))
}

pub async fn route_delete<R: ResourceService>(
    path_components: &R::PathComponents,
    principal: R::Principal,
    resource_service: &R,
    no_trash: bool,
    if_match: Option<IfMatch>,
    if_none_match: Option<IfNoneMatch>,
) -> Result<(), R::Error> {
    let resource = resource_service.get_resource(path_components).await?;

    let privileges = resource.get_user_privileges(&principal)?;
    if !privileges.has(&UserPrivilege::Write) {
        return Err(Error::Unauthorized.into());
    }

    if let Some(if_match) = if_match {
        if !resource.satisfies_if_match(&if_match) {
            // Precondition failed
            return Err(crate::Error::PreconditionFailed.into());
        }
    }
    if let Some(if_none_match) = if_none_match {
        if resource.satisfies_if_none_match(&if_none_match) {
            // Precondition failed
            return Err(crate::Error::PreconditionFailed.into());
        }
    }
    resource_service
        .delete_resource(path_components, !no_trash)
        .await?;
    Ok(())
}
