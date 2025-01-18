use crate::calendar::resource::CalendarResource;
use crate::Error;
use actix_web::http::header;
use actix_web::web::{Data, Path};
use actix_web::{HttpRequest, HttpResponse};
use rustical_dav::privileges::UserPrivilege;
use rustical_dav::push::PushRegister;
use rustical_dav::resource::Resource;
use rustical_store::auth::User;
use rustical_store::{CalendarStore, Subscription, SubscriptionStore};
use rustical_xml::XmlDocument;
use tracing::instrument;
use tracing_actix_web::RootSpan;

#[instrument(parent = root_span.id(), skip(store, subscription_store, root_span, req))]
pub async fn route_post<C: CalendarStore, S: SubscriptionStore>(
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
    let calendar_resource = CalendarResource {
        cal: calendar,
        read_only: true,
    };

    if !calendar_resource
        .get_user_privileges(&user)?
        .has(&UserPrivilege::Read)
    {
        return Err(Error::Unauthorized);
    }

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
        topic: calendar_resource.cal.push_topic,
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
