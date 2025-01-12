use crate::Error;
use actix_web::http::header;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use rustical_store::auth::User;
use rustical_store::{CalendarStore, Subscription, SubscriptionStore};
use rustical_xml::{XmlDeserialize, XmlDocument, XmlRootTag};
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
struct WebPushSubscription {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    push_resource: String,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
struct SubscriptionElement {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub web_push_subscription: WebPushSubscription,
}

#[derive(XmlDeserialize, XmlRootTag, Clone, Debug, PartialEq)]
#[xml(root = b"push-register")]
#[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
struct PushRegister {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    subscription: SubscriptionElement,
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    expires: Option<String>,
}

#[instrument(parent = root_span.id(), skip(store, subscription_store, root_span, req))]
pub async fn route_post<C: CalendarStore + ?Sized, S: SubscriptionStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    store: Data<C>,
    subscription_store: Data<S>,
    root_span: RootSpan,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (principal, cal_id) = path.into_inner();
    if principal != user.id {
        return Err(Error::Unauthorized);
    }

    let calendar = store.get_calendar(&principal, &cal_id).await?;
    let request = PushRegister::parse_str(&body)?;
    let sub_id = uuid::Uuid::new_v4().to_string();

    let expires = if let Some(expires) = request.expires {
        chrono::DateTime::parse_from_rfc2822(&expires)
            .map_err(|err| crate::Error::Other(err.into()))?
    } else {
        chrono::Utc::now().fixed_offset() + chrono::Duration::weeks(1)
    };

    let subscription = Subscription {
        id: sub_id.to_owned(),
        push_resource: request
            .subscription
            .web_push_subscription
            .push_resource
            .to_owned(),
        topic: calendar.push_topic,
        expiration: expires.naive_local(),
    };
    subscription_store.upsert_subscription(subscription).await?;

    let location = req
        .resource_map()
        .url_for(&req, "subscription", &[sub_id])
        .unwrap();

    Ok(HttpResponse::Created()
        .append_header((header::LOCATION, location.to_string()))
        .append_header((header::EXPIRES, expires.to_rfc2822()))
        .finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_push_register() {
        let push_register = PushRegister::parse_str(
            r#"
            <?xml version="1.0" encoding="utf-8" ?>
            <push-register xmlns="https://bitfire.at/webdav-push">
                <subscription>
                    <web-push-subscription>
                        <push-resource>https://up.example.net/yohd4yai5Phiz1wi</push-resource>
                    </web-push-subscription>
                </subscription>
                <expires>Wed, 20 Dec 2023 10:03:31 GMT</expires>
            </push-register>
    "#,
        )
        .unwrap();
        assert_eq!(
            push_register,
            PushRegister {
                subscription: SubscriptionElement {
                    web_push_subscription: WebPushSubscription {
                        push_resource: "https://up.example.net/yohd4yai5Phiz1wi".to_owned()
                    }
                },
                expires: Some("Wed, 20 Dec 2023 10:03:31 GMT".to_owned())
            }
        )
    }
}
