use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use tracing::instrument;

use crate::{
    header::{Depth, Overwrite},
    resource::ResourceService,
};

#[instrument(skip(_path, _resource_service,))]
pub(crate) async fn axum_route_copy<R: ResourceService>(
    Path(_path): Path<R::PathComponents>,
    State(_resource_service): State<R>,
    depth: Option<Depth>,
    principal: R::Principal,
    overwrite: Overwrite,
) -> Result<Response, R::Error> {
    // TODO: Actually implement, but to be WebDAV-compliant we must at least support this route but
    // can return a 403 error
    let _depth = depth.unwrap_or(Depth::Infinity);
    Ok(StatusCode::FORBIDDEN.into_response())
}
