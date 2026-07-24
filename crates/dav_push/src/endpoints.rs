use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::delete,
};
use http::StatusCode;
use std::sync::Arc;

use crate::SubscriptionStore;

async fn handle_delete<S: SubscriptionStore>(
    State(store): State<Arc<S>>,
    Path(id): Path<String>,
) -> Result<Response, rustical_store::Error> {
    store.delete_subscription(&id).await?;
    Ok((StatusCode::NO_CONTENT, "Unregistered").into_response())
}

pub fn subscription_service<S: SubscriptionStore>(sub_store: Arc<S>) -> Router {
    Router::new()
        .route("/push_subscription/{id}", delete(handle_delete::<S>))
        .with_state(sub_store)
}
