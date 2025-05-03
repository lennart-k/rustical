use crate::Error;
use actix_web::http::header;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use rustical_dav_push::register::PushRegister;
use rustical_store::auth::User;
use rustical_store::{AddressbookStore, Subscription, SubscriptionStore};
use rustical_xml::XmlDocument;
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[instrument(parent = root_span.id(), skip(store, subscription_store, root_span, req))]
pub async fn route_post<A: AddressbookStore, S: SubscriptionStore>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    store: Data<A>,
    subscription_store: Data<S>,
    root_span: RootSpan,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let (principal, addressbook_id) = path.into_inner();
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let addressbook = store
        .get_addressbook(&principal, &addressbook_id, false)
        .await?;
    let request = PushRegister::parse_str(&body)?;
    let sub_id = uuid::Uuid::new_v4().to_string();

    let expires = if let Some(expires) = request.expires {
        chrono::DateTime::parse_from_rfc2822(&expires).map_err(Error::from)?
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
        topic: addressbook.push_topic,
        expiration: expires.naive_local(),
        public_key: request
            .subscription
            .web_push_subscription
            .subscription_public_key
            .key,
        public_key_type: request
            .subscription
            .web_push_subscription
            .subscription_public_key
            .ty,
        auth_secret: request.subscription.web_push_subscription.auth_secret,
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
