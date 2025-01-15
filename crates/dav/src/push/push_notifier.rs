use crate::xml::multistatus::PropstatElement;
use actix_web::http::StatusCode;
use rustical_store::{CollectionOperation, CollectionOperationType, SubscriptionStore};
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tracing::{error, info};

#[derive(XmlSerialize, Debug)]
struct PushMessageProp {
    #[xml(ns = "crate::namespace::NS_DAV")]
    topic: String,
    #[xml(ns = "crate::namespace::NS_DAV")]
    sync_token: Option<String>,
}

#[derive(XmlSerialize, XmlRootTag, Debug)]
#[xml(root = b"push-message", ns = "crate::namespace::NS_DAVPUSH")]
#[xml(ns_prefix(crate::namespace::NS_DAVPUSH = b"", crate::namespace::NS_DAV = b"D",))]
struct PushMessage {
    #[xml(ns = "crate::namespace::NS_DAV")]
    propstat: PropstatElement<PushMessageProp>,
}

pub async fn push_notifier(
    mut recv: Receiver<CollectionOperation>,
    sub_store: Arc<impl SubscriptionStore>,
) {
    while let Some(message) = recv.recv().await {
        if let Ok(subscribers) = sub_store.get_subscriptions(&message.topic).await {
            let status = match message.r#type {
                CollectionOperationType::Object => StatusCode::OK,
                CollectionOperationType::Delete => StatusCode::NOT_FOUND,
            };
            let push_message = PushMessage {
                propstat: PropstatElement {
                    prop: PushMessageProp {
                        topic: message.topic,
                        sync_token: message.sync_token,
                    },
                    status,
                },
            };
            let mut output: Vec<_> = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".into();
            let mut writer = quick_xml::Writer::new_with_indent(&mut output, b' ', 4);
            if let Err(err) = push_message.serialize_root(&mut writer) {
                error!("Could not serialize push message: {}", err);
                continue;
            }
            let payload = String::from_utf8(output).unwrap();
            for subscriber in subscribers {
                info!(
                    "Sending a push message to {}: {}",
                    subscriber.push_resource, payload
                );
                let client = reqwest::Client::new();
                if let Err(err) = client
                    .post(subscriber.push_resource)
                    .body(payload.to_owned())
                    .send()
                    .await
                {
                    error!("{err}");
                }
            }
        }
    }
}
