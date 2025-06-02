mod delete;
mod propfind;
mod proppatch;

pub(crate) use delete::route_delete;
pub(crate) use propfind::route_propfind;
pub(crate) use proppatch::route_proppatch;

#[cfg(feature = "actix")]
pub(crate) use delete::actix_route_delete;

#[cfg(feature = "actix")]
pub(crate) use propfind::actix_route_propfind;
