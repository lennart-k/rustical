#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
mod extension;
mod prop;
pub mod register;
use chrono::Utc;
pub use extension::*;
pub use prop::*;
use reqwest::Url;
use rustical_store::{CollectionOperation, CollectionOperationInfo};
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::mpsc::Receiver;
use tracing::{error, info, warn};
use web_push::{
    Claims, ContentEncoding, VapidSignatureBuilder, WebPushClient, WebPushMessage,
    WebPushMessageBuilder,
};

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
    sync_token: String,
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

#[derive(Debug, thiserror::Error)]
pub enum DavPushError {
    #[error(transparent)]
    StoreError(#[from] rustical_store::Error),
    #[error("Could not serialize push message: {0}")]
    XmlError(#[from] std::io::Error),
    #[error("Web Push error: {0}")]
    WebPushError(#[from] web_push::WebPushError),
}

#[derive(Debug)]
pub struct DavPushService<DP: SubscriptionStore> {
    allowed_push_servers: Option<Vec<String>>,
    dav_push_store: Arc<DP>,
    client: reqwest::Client,
    vapid_key: VapidKeypair,
}

impl<DP: SubscriptionStore> DavPushService<DP> {
    pub fn new(
        allowed_push_servers: Option<Vec<String>>,
        dav_push_store: Arc<DP>,
        vapid_key: VapidKeypair,
    ) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .read_timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            allowed_push_servers,
            dav_push_store,
            client,
            vapid_key,
        })
    }

    /// This is the entrypoint of the service
    pub async fn notifier_loop(&self, mut recv: Receiver<CollectionOperation>) {
        loop {
            // Make sure we don't flood the subscribers
            tokio::time::sleep(Duration::from_secs(10)).await;
            let mut operations = Vec::new();
            recv.recv_many(&mut operations, 100).await;

            // Right now we just have to show the latest content update by topic
            // This might become more complicated in the future depending on what kind of updates
            // we add

            let changes: Vec<(String, String)> = operations
                .into_iter()
                .filter_map(|operation| {
                    if let CollectionOperationInfo::Content { sync_token } = operation.data {
                        Some((operation.topic, sync_token))
                    } else {
                        None
                    }
                })
                // Deduplicate by topic, keep latest change
                .collect::<HashMap<_, _>>()
                .into_iter()
                .collect();

            for (topic, synctoken) in changes {
                if let Err(err) = self.send_update(topic, synctoken).await {
                    error!("{err}");
                }
            }
        }
    }

    async fn send_update(&self, topic: String, sync_token: String) -> Result<(), DavPushError> {
        let subscriptions = self.dav_push_store.get_subscriptions(&topic).await?;
        if subscriptions.is_empty() {
            return Ok(());
        }

        let push_message = PushMessage {
            topic,
            content_update: Some(ContentUpdate { sync_token }),
        };

        let payload = push_message.serialize_to_string()?;

        for subscription in subscriptions {
            if subscription.is_expired(&Utc::now()) {
                info!(
                    "Deleting subscription {} on topic {} because it is expired",
                    subscription.id, subscription.topic
                );
                self.try_delete_subscription(&subscription.id).await;
                return Ok(());
            }

            let Ok(url) = Url::parse(&subscription.push_resource) else {
                warn!(
                    "Deleting subscription {} on topic {} because of invalid URL",
                    subscription.id, subscription.topic
                );
                self.try_delete_subscription(&subscription.id).await;
                return Ok(());
            };

            if let Some(allowed_push_servers) = &self.allowed_push_servers
                && !allowed_push_servers.contains(&url.origin().unicode_serialization())
            {
                return Ok(());
            }

            if let Err(err) = self
                .send_payload_to_subscriber(subscription, &payload)
                .await
            {
                error!("An error occured sending a WebDAV Push notification: {err}");
            }
        }

        Ok(())
    }
    async fn send_payload_to_subscriber(
        &self,
        subscription: Subscription,
        payload: &str,
    ) -> Result<(), DavPushError> {
        let message = build_message(self.vapid_key.clone(), subscription, payload)?;
        let client = web_push::ReqwestWebPushClient::from_client(self.client.clone());
        client.send(message).await?;
        Ok(())
    }

    // Try to delete a subscription ignoring any errors
    async fn try_delete_subscription(&self, sub_id: &str) {
        if let Err(err) = self.dav_push_store.delete_subscription(sub_id).await {
            error!("Error deleting subsciption: {err}");
        }
    }
}

fn build_message(
    vapid_key: VapidKeypair,
    subscription: Subscription,
    payload: &str,
) -> Result<WebPushMessage, DavPushError> {
    let subscription_info: web_push::SubscriptionInfo = subscription.into();
    let signature =
        VapidSignatureBuilder::from_ec(vapid_key.0, &subscription_info).build(Claims::new())?;

    Ok(WebPushMessageBuilder::new(&subscription_info)
        .payload(ContentEncoding::Aes128Gcm, payload.as_bytes())
        .vapid_signature(signature)
        .build()?)
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
    use insta::assert_debug_snapshot;

    use crate::{Subscription, VapidKeypair, build_message, vapid};

    #[test]
    fn test_build_message_valid() {
        let vapid_key = VapidKeypair::from_pem(vapid::tests::PRIVATE_KEY_PEM).unwrap();
        let subscription = Subscription {
            topic: "d0be5c46-36a4-4019-a0f9-e47a3e386096".to_string(),
            id: "adaf9631-4497-47a9-8550-cd5fee203448".to_string(),
            expiration: NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
                NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            ),
            auth_secret: "VG60BKYQf6ZFIEIfLB5CdQ".to_string(),
            push_resource: "https://ntfy.example.com/upL00-v4L3SGM2".to_string(),
            public_key: "BNfY5sCh5FqswvkO3KNHeR3vjzOSV0sSmvwS0mePb86ve6CBaoiyQYuX7PIJ0rvDLvOIa2HDWFxnDCyMB7HrqlE".to_string(),
            public_key_type: "p256dh".to_string(),
        };
        let payload = "asd";
        let message = build_message(vapid_key, subscription, payload).unwrap();
        assert_debug_snapshot!(message.endpoint, @"https://ntfy.example.com/upL00-v4L3SGM2");
        assert_debug_snapshot!(message.ttl, @"2419200");
        assert_debug_snapshot!(message.urgency, @"None");
        assert_debug_snapshot!(message.topic, @"None");
    }
}
