#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
mod extension;
mod prop;
pub mod register;
use chrono::Utc;
use derive_more::Constructor;
pub use extension::*;
pub use prop::*;
use reqwest::Url;
use rustical_store::{CollectionOperation, CollectionOperationInfo};
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::mpsc::Receiver;
use tracing::{error, info, warn};
use web_push::{ContentEncoding, VapidSignatureBuilder, WebPushClient, WebPushMessageBuilder};

mod endpoints;
pub use endpoints::subscription_service;

pub(crate) mod vapid;
pub use vapid::{VapidError, VapidKeypair, VapidPublicKey, VapidPublicKeyB64};

mod store;
pub use store::*;

mod subscription;
pub use subscription::*;

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
pub struct DavPushController<DP: DavPushStore> {
    allowed_push_servers: Option<Vec<String>>,
    sub_store: Arc<DP>,
}

impl<DP: DavPushStore> DavPushController<DP> {
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
                    latest_messages.insert(message.topic.clone(), message);
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
        let vapid_key = match self.sub_store.get_vapid_keypair().await {
            Ok(key) => key,
            Err(err) => {
                error!("{err}");
                return;
            }
        };

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

        for subscription in subscriptions {
            if subscription.is_expired(&Utc::now()) {
                info!(
                    "Deleting subscription {} on topic {} because it is expired",
                    subscription.id, subscription.topic
                );
                self.try_delete_subscription(&subscription.id).await;
                continue;
            }

            if let Some(allowed_push_servers) = &self.allowed_push_servers {
                if let Ok(url) = Url::parse(&subscription.push_resource) {
                    let origin = url.origin().unicode_serialization();
                    if !allowed_push_servers.contains(&origin) {
                        warn!(
                            "Deleting subscription {} on topic {} because the endpoint is not in the list of allowed push servers",
                            subscription.id, subscription.topic
                        );
                        self.try_delete_subscription(&subscription.id).await;
                        continue;
                    }
                } else {
                    warn!(
                        "Deleting subscription {} on topic {} because of invalid URL",
                        subscription.id, subscription.topic
                    );
                    self.try_delete_subscription(&subscription.id).await;
                    continue;
                }
            }

            let subscription_info: web_push::SubscriptionInfo = subscription.into();

            let mut message_builder = WebPushMessageBuilder::new(&subscription_info);
            message_builder.set_payload(ContentEncoding::Aes128Gcm, payload.as_bytes());
            let signature = VapidSignatureBuilder::from_ec(vapid_key.0.clone(), &subscription_info)
                .build()
                .unwrap();
            message_builder.set_vapid_signature(signature);
            let message = message_builder.build().unwrap();

            let client = web_push::ReqwestWebPushClient::new().unwrap();
            if let Err(err) = client.send(message).await {
                error!("An error occured sending out a push notification: {err}");
                // if err.is_permament_error() {
                //     warn!(
                //         "Deleting subscription {} on topic {}",
                //         &subscription_id, subscription_topic
                //     );
                //     self.try_delete_subscription(&subscription_id).await;
                // }
            }
        }
    }

    async fn try_delete_subscription(&self, sub_id: &str) {
        if let Err(err) = self.sub_store.delete_subscription(sub_id).await {
            error!("Error deleting subsciption: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    // #[tokio::test]
    // async fn test_ntfy_request() {
    //     let (keypair, auth_secret) = generate_keypair_and_auth_secret().unwrap();
    //     let auth_secret = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(auth_secret);
    //     let public_key =
    //         base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(keypair.pub_as_raw().unwrap());
    //
    //     send_payload(
    //         "hello",
    //         &Subscription {
    //             id: "asd".to_string(),
    //             topic: "asd".to_string(),
    //             expiration: NaiveDateTime::MAX,
    //             push_resource: "https://ntfy.sh/upL00-v4L3SGM2".to_string(),
    //             public_key,
    //             public_key_type: "p256dh".to_string(),
    //             auth_secret,
    //         },
    //     )
    //     .await
    //     .unwrap();
    // }
}
