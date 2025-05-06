use actix_web::http::StatusCode;
use reqwest::{
    Method, Request,
    header::{self, HeaderName, HeaderValue},
};
use rustical_dav::xml::multistatus::PropstatElement;
use rustical_store::{CollectionOperation, CollectionOperationType, SubscriptionStore};
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use std::{str::FromStr, sync::Arc};
use tokio::sync::mpsc::Receiver;
use tracing::{error, info, warn};
// use web_push::{SubscriptionInfo, WebPushMessage, WebPushMessageBuilder};

#[derive(XmlSerialize, Debug)]
struct PushMessageProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    topic: String,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    sync_token: Option<String>,
}

#[derive(XmlSerialize, XmlRootTag, Debug)]
#[xml(root = b"push-message", ns = "rustical_dav::namespace::NS_DAVPUSH")]
#[xml(ns_prefix(
    rustical_dav::namespace::NS_DAVPUSH = b"",
    rustical_dav::namespace::NS_DAV = b"D",
))]
struct PushMessage {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    propstat: PropstatElement<PushMessageProp>,
}

// pub fn build_request(message: WebPushMessage) -> Request {
//     // A little janky :)
//     let url = reqwest::Url::from_str(&message.endpoint.to_string()).unwrap();
//     let mut builder = Request::new(Method::POST, url);
//
//     if let Some(topic) = message.topic {
//         builder
//             .headers_mut()
//             .insert("Topic", HeaderValue::from_str(topic.as_str()).unwrap());
//     }
//
//     if let Some(payload) = message.payload {
//         builder.headers_mut().insert(
//             header::CONTENT_ENCODING,
//             HeaderValue::from_static(payload.content_encoding.to_str()),
//         );
//         builder.headers_mut().insert(
//             header::CONTENT_TYPE,
//             HeaderValue::from_static("application/octet-stream"),
//         );
//
//         for (k, v) in payload.crypto_headers.into_iter() {
//             let v: &str = v.as_ref();
//             builder.headers_mut().insert(
//                 HeaderName::from_static(k),
//                 HeaderValue::from_str(&v).unwrap(),
//             );
//         }
//
//         *builder.body_mut() = Some(reqwest::Body::from(payload.content));
//     }
//     builder
// }

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
        // for subscriber in subscribers {
        //     let push_resource = subscriber.push_resource;
        //
        //     let sub_info = SubscriptionInfo {
        //         endpoint: push_resource.to_owned(),
        //         keys: web_push::SubscriptionKeys {
        //             p256dh: subscriber.public_key,
        //             auth: subscriber.auth_secret,
        //         },
        //     };
        //     let mut builder = WebPushMessageBuilder::new(&sub_info);
        //     builder.set_payload(web_push::ContentEncoding::Aes128Gcm, payload.as_bytes());
        //     let push_message = builder.build().unwrap();
        //     let request = build_request(push_message);
        //
        //     let allowed = if let Some(allowed_push_servers) = &allowed_push_servers {
        //         if let Ok(resource_url) = reqwest::Url::parse(&push_resource) {
        //             let origin = resource_url.origin().ascii_serialization();
        //             allowed_push_servers
        //                 .iter()
        //                 .any(|allowed_push_server| allowed_push_server == &origin)
        //         } else {
        //             warn!("Invalid push url: {push_resource}");
        //             false
        //         }
        //     } else {
        //         true
        //     };
        //
        //     if allowed {
        //         info!("Sending a push message to {}: {}", push_resource, payload);
        //         if let Err(err) = client.execute(request).await {
        //             error!("{err}");
        //         }
        //     } else {
        //         warn!(
        //             "Not sending a push notification to {} since it's not allowed in dav_push::allowed_push_servers",
        //             push_resource
        //         );
        //     }
        // }
    }
}
