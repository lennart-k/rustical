use crate::Error;
use crate::calendar::CalendarResourceService;
use crate::calendar::resource::CalendarResource;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderValue, StatusCode, header};
use rustical_dav::privileges::UserPrivilege;
use rustical_dav::resource::Resource;
use rustical_dav_push::register::PushRegister;
use rustical_store::auth::Principal;
use rustical_store::{CalendarStore, Subscription, SubscriptionStore};
use rustical_xml::XmlDocument;
use tracing::instrument;

#[instrument(skip(resource_service))]
pub async fn route_post<C: CalendarStore, S: SubscriptionStore>(
    Path((principal, cal_id)): Path<(String, String)>,
    user: Principal,
    State(resource_service): State<CalendarResourceService<C, S>>,
    body: String,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let calendar = resource_service
        .cal_store
        .get_calendar(&principal, &cal_id, false)
        .await?;
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
        topic: calendar_resource.cal.push_topic,
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
    resource_service
        .sub_store
        .upsert_subscription(subscription)
        .await?;

    // TODO: make nicer
    let location = format!("/push_subscription/{sub_id}");
    Ok((
        StatusCode::CREATED,
        HeaderMap::from_iter([
            (header::LOCATION, HeaderValue::from_str(&location).unwrap()),
            (
                header::EXPIRES,
                HeaderValue::from_str(&expires.to_rfc2822()).unwrap(),
            ),
        ]),
    )
        .into_response())
}
