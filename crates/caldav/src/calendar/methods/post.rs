use actix_web::{web::Data, web::Path, Responder};
use rustical_store::{auth::User, CalendarStore};
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[instrument(parent = root_span.id(), skip(store, root_span))]
pub async fn route_post<C: CalendarStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    store: Data<C>,
    root_span: RootSpan,
) -> impl Responder {
    "asd"
}
