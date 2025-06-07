mod delete;
mod propfind;
mod proppatch;

#[cfg(feature = "actix")]
pub(crate) use delete::actix_route_delete;
#[cfg(feature = "axum")]
pub(crate) use delete::axum_route_delete;

#[cfg(feature = "actix")]
pub(crate) use propfind::actix_route_propfind;
#[cfg(feature = "axum")]
pub(crate) use propfind::axum_route_propfind;

#[cfg(feature = "actix")]
pub(crate) use proppatch::actix_route_proppatch;
#[cfg(feature = "axum")]
pub(crate) use proppatch::axum_route_proppatch;
