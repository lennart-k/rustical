#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
mod extension;
mod prop;
pub mod register;
use base64::Engine;
use derive_more::Constructor;
pub use extension::*;
use http::{HeaderValue, Method, header};
pub use prop::*;
use reqwest::{Body, Url};
use rustical_store::{
    CollectionOperation, CollectionOperationInfo, Subscription, SubscriptionStore,
};
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::mpsc::Receiver;
use tracing::{error, warn};

mod endpoints;
pub use endpoints::subscription_service;

#[derive(XmlSerialize, Debug)]
pub struct ContentUpdate {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    sync_token: Option<String>,
}

#[derive(XmlSerialize, XmlRootTag, Debug)]
#[xml(root = "push-message", ns = "rustical_dav::namespace::NS_DAVPUSH")]
#[xml(ns_prefix(
    rustical_dav::namespace::NS_DAVPUSH = "",
    rustical_dav::namespace::NS_DAV = "D",
))]
struct PushMessage {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    topic: String,
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    content_update: Option<ContentUpdate>,
}

#[derive(Debug, Constructor)]
pub struct DavPushController<S: SubscriptionStore> {
    allowed_push_servers: Option<Vec<String>>,
    sub_store: Arc<S>,
}

impl<S: SubscriptionStore> DavPushController<S> {
    pub async fn notifier(&self, mut recv: Receiver<CollectionOperation>) {
        loop {
            // Make sure we don't flood the subscribers
            tokio::time::sleep(Duration::from_secs(10)).await;
            let mut messages = vec![];
            recv.recv_many(&mut messages, 100).await;

            // Right now we just have to show the latest content update by topic
            // This might become more complicated in the future depending on what kind of updates
            // we add
            let mut latest_messages = HashMap::new();
            for message in messages {
                if matches!(message.data, CollectionOperationInfo::Content { .. }) {
                    latest_messages.insert(message.topic.to_string(), message);
                }
            }
            let messages = latest_messages.into_values();

            for message in messages {
                self.send_message(message).await;
            }
        }
    }

    #[allow(clippy::cognitive_complexity)]
    async fn send_message(&self, message: CollectionOperation) {
        let subscriptions = match self.sub_store.get_subscriptions(&message.topic).await {
            Ok(subs) => subs,
            Err(err) => {
                error!("{err}");
                return;
            }
        };

        if subscriptions.is_empty() {
            return;
        }

        if matches!(message.data, CollectionOperationInfo::Delete) {
            // Collection has been deleted, but we cannot handle that
            return;
        }

        let content_update = if let CollectionOperationInfo::Content { sync_token } = message.data {
            Some(ContentUpdate {
                sync_token: Some(sync_token),
            })
        } else {
            None
        };

        let push_message = PushMessage {
            topic: message.topic,
            content_update,
        };

        let payload = match push_message.serialize_to_string() {
            Ok(payload) => payload,
            Err(err) => {
                error!("Could not serialize push message: {}", err);
                return;
            }
        };

        for subsciption in subscriptions {
            if let Some(allowed_push_servers) = &self.allowed_push_servers {
                if let Ok(url) = Url::parse(&subsciption.push_resource) {
                    let origin = url.origin().unicode_serialization();
                    if !allowed_push_servers.contains(&origin) {
                        warn!(
                            "Deleting subscription {} on topic {} because the endpoint is not in the list of allowed push servers",
                            subsciption.id, subsciption.topic
                        );
                        self.try_delete_subscription(&subsciption.id).await;
                    }
                } else {
                    warn!(
                        "Deleting subscription {} on topic {} because of invalid URL",
                        subsciption.id, subsciption.topic
                    );
                    self.try_delete_subscription(&subsciption.id).await;
                }
            }

            if let Err(err) = self.send_payload(&payload, &subsciption).await {
                error!("An error occured sending out a push notification: {err}");
                if err.is_permament_error() {
                    warn!(
                        "Deleting subscription {} on topic {}",
                        subsciption.id, subsciption.topic
                    );
                    self.try_delete_subscription(&subsciption.id).await;
                }
            }
        }
    }

    async fn try_delete_subscription(&self, sub_id: &str) {
        if let Err(err) = self.sub_store.delete_subscription(sub_id).await {
            error!("Error deleting subsciption: {err}");
        }
    }

    async fn send_payload(
        &self,
        payload: &str,
        subsciption: &Subscription,
    ) -> Result<(), NotifierError> {
        if subsciption.public_key_type != "p256dh" {
            return Err(NotifierError::InvalidPublicKeyType(
                subsciption.public_key_type.to_string(),
            ));
        }
        let endpoint = subsciption.push_resource.parse().map_err(|_| {
            NotifierError::InvalidEndpointUrl(subsciption.push_resource.to_string())
        })?;
        let ua_public = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&subsciption.public_key)
            .map_err(|_| NotifierError::InvalidKeyEncoding)?;
        let auth_secret = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&subsciption.auth_secret)
            .map_err(|_| NotifierError::InvalidKeyEncoding)?;

        let client = reqwest::ClientBuilder::new()
            .build()
            .map_err(NotifierError::from)?;

        let payload = ece::encrypt(&ua_public, &auth_secret, payload.as_bytes())?;

        let mut request = reqwest::Request::new(Method::POST, endpoint);
        *request.body_mut() = Some(Body::from(payload));
        let hdrs = request.headers_mut();
        hdrs.insert(
            header::CONTENT_ENCODING,
            HeaderValue::from_static("aes128gcm"),
        );
        hdrs.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );
        hdrs.insert("TTL", HeaderValue::from(60));
        client.execute(request).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
enum NotifierError {
    #[error("Invalid public key type: {0}")]
    InvalidPublicKeyType(String),
    #[error("Invalid endpoint URL: {0}")]
    InvalidEndpointUrl(String),
    #[error("Invalid key encoding")]
    InvalidKeyEncoding,
    #[error(transparent)]
    EceError(#[from] ece::Error),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

impl NotifierError {
    // Decide whether the error should cause the subscription to be removed
    pub const fn is_permament_error(&self) -> bool {
        match self {
            Self::InvalidPublicKeyType(_)
            | Self::InvalidEndpointUrl(_)
            | Self::InvalidKeyEncoding => true,
            Self::EceError(err) => matches!(
                err,
                ece::Error::InvalidAuthSecret | ece::Error::InvalidKeyLength
            ),
            Self::ReqwestError(_) => false,
        }
    }
}
