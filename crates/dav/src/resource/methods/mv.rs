use crate::{
    header::{Depth, Overwrite},
    resource::ResourceService,
};
use axum::{
    extract::{MatchedPath, Path, State},
    response::{IntoResponse, Response},
};
use http::{HeaderMap, StatusCode, Uri};
use matchit_serde::ParamsDeserializer;
use serde::Deserialize;
use tracing::instrument;

#[instrument(skip(path, resource_service,))]
pub(crate) async fn axum_route_move<R: ResourceService>(
    Path(path): Path<R::PathComponents>,
    State(resource_service): State<R>,
    depth: Option<Depth>,
    principal: R::Principal,
    Overwrite(overwrite): Overwrite,
    matched_path: MatchedPath,
    header_map: HeaderMap,
) -> Result<Response, R::Error> {
    let destination = header_map
        .get("Destination")
        .ok_or(crate::Error::Forbidden)?
        .to_str()
        .map_err(|_| crate::Error::Forbidden)?;
    let destination_uri: Uri = destination.parse().map_err(|_| crate::Error::Forbidden)?;
    // TODO: Check that host also matches
    let destination = destination_uri.path();

    let mut router = matchit::Router::new();
    router.insert(matched_path.as_str(), ()).unwrap();
    if let Ok(matchit::Match { params, .. }) = router.at(destination) {
        let params =
            matchit_serde::Params::try_from(&params).map_err(|_| crate::Error::Forbidden)?;
        let dest_path = R::PathComponents::deserialize(&ParamsDeserializer::new(params))
            .map_err(|_| crate::Error::Forbidden)?;

        if resource_service
            .copy_resource(&path, &dest_path, &principal, overwrite)
            .await?
        {
            // Overwritten
            Ok(StatusCode::NO_CONTENT.into_response())
        } else {
            // Not overwritten
            Ok(StatusCode::CREATED.into_response())
        }
    } else {
        Ok(StatusCode::FORBIDDEN.into_response())
    }
}
