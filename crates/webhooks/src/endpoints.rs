use axum::{
    Router,
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::{delete, post, get},
};
use http::StatusCode;
use rustical_store::{ WebhookSubscriptionStore};
use std::sync::Arc;
use serde_json::json;
use serde::Deserialize;
use axum::extract::Json;
use rustical_types::WebhookSubscription;
use uuid::Uuid;

async fn handle_delete<S: WebhookSubscriptionStore>(
    State(store): State<Arc<S>>,
    Path(id): Path<String>,
) -> Result<Response, rustical_store::Error> {
    store.delete_subscription(&id).await?;
    Ok((StatusCode::NO_CONTENT, "Unregistered").into_response())
}


#[derive(Deserialize)]
struct UpsertPayload {
    id: Option<String>,
    target_url: String,
    resource_type: String,
    resource_id: String,
    secret_key: Option<String>,
}

async fn handle_upsert<S: WebhookSubscriptionStore>(
    State(store): State<Arc<S>>,
    Json(payload): Json<UpsertPayload>,
) -> Result<Response, rustical_store::Error> {
    let id = payload.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let subscription = WebhookSubscription {
        id: id.clone(),
        target_url: payload.target_url,
        resource_type: payload.resource_type,
        resource_id: payload.resource_id,
        secret_key: payload.secret_key,
    };
    let already_exists = store.upsert_subscription(subscription).await?;
    let status = if already_exists { StatusCode::OK } else { StatusCode::CREATED };
    let body = json!({
        "id": id,
        "updated": already_exists,
        "status": if already_exists { "updated" } else { "created" }
    });
    Ok((status, Json(body)).into_response())
}

async fn handle_get<S: WebhookSubscriptionStore>(
    State(store): State<Arc<S>>,
    Path(id): Path<String>,
) -> Result<Response, rustical_store::Error> {
    let sub = store.get_subscription(&id).await?;
    Ok((StatusCode::OK, Json(sub)).into_response())
}

async fn handle_list<S: WebhookSubscriptionStore>(
    State(store): State<Arc<S>>,
    Path((resource_type, resource_id)): Path<(String, String)>,
) -> Result<Response, rustical_store::Error> {
    let subs = store.get_subscriptions(&resource_type, &resource_id).await?;
    Ok((StatusCode::OK, Json(json!({"subscriptions": subs}))).into_response())
}

// public function to create the webhook subscription router
pub fn webhook_subscription_router<S: WebhookSubscriptionStore>(store: Arc<S>) -> Router {
    Router::new()
        .route("/webhooks/subscriptions/upsert", post(handle_upsert::<S>))
        .route("/webhooks/subscriptions/delete/{id}", delete(handle_delete::<S>))
        .route("/webhooks/subscriptions/id/{id}", get(handle_get::<S>))
        .route("/webhooks/subscriptions/{resource_type}/{resource_id}", get(handle_list::<S>))
        .with_state(store)
}