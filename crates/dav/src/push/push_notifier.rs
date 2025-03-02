use crate::xml::multistatus::PropstatElement;
use axum::http::StatusCode;
use rustical_store::{CollectionOperation, CollectionOperationType, SubscriptionStore};
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tracing::{error, info, warn};

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
    allowed_push_servers: Option<Vec<String>>,
    mut recv: Receiver<CollectionOperation>,
    sub_store: Arc<impl SubscriptionStore>,
) {
    let client = reqwest::Client::new();

    while let Some(message) = recv.recv().await {
        let subscribers = match sub_store.get_subscriptions(&message.topic).await {
            Ok(subs) => subs,
            Err(err) => {
                error!("{err}");
                continue;
            }
        };

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
            let push_resource = subscriber.push_resource;
            let allowed = if let Some(allowed_push_servers) = &allowed_push_servers {
                if let Ok(resource_url) = reqwest::Url::parse(&push_resource) {
                    let origin = resource_url.origin().ascii_serialization();
                    allowed_push_servers
                        .iter()
                        .any(|allowed_push_server| allowed_push_server == &origin)
                } else {
                    warn!("Invalid push url: {push_resource}");
                    false
                }
            } else {
                true
            };

            if allowed {
                info!("Sending a push message to {}: {}", push_resource, payload);
                if let Err(err) = client
                    .post(push_resource)
                    .body(payload.to_owned())
                    .send()
                    .await
                {
                    error!("{err}");
                }
            } else {
                warn!("Not sending a push notification to {} since it's not allowed in dav_push::allowed_push_servers", push_resource);
            }
        }
    }
}
