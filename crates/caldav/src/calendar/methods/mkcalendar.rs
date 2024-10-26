use crate::Error;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use rustical_store::auth::User;
use rustical_store::model::Calendar;
use rustical_store::CalendarStore;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    #[serde(rename = "C:calendar", alias = "calendar")]
    calendar: Option<()>,
    collection: Option<()>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct CalendarComponentElement {
    #[serde(rename = "@name")]
    name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedCalendarComponentSetElement {
    #[serde(rename = "$value")]
    comp: Vec<CalendarComponentElement>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct MkcolCalendarProp {
    resourcetype: Option<Resourcetype>,
    displayname: Option<String>,
    calendar_description: Option<String>,
    calendar_color: Option<String>,
    order: Option<i64>,
    calendar_timezone: Option<String>,
    supported_calendar_component_set: Option<SupportedCalendarComponentSetElement>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PropElement<T: Serialize> {
    prop: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
#[serde(rename = "mkcalendar")]
struct MkcalendarRequest {
    set: PropElement<MkcolCalendarProp>,
}

pub async fn route_mkcalendar<C: CalendarStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    store: Data<RwLock<C>>,
) -> Result<HttpResponse, Error> {
    let (principal, cal_id) = path.into_inner();
    if principal != user.id {
        return Err(Error::Unauthorized);
    }

    let request: MkcalendarRequest = quick_xml::de::from_str(&body)?;
    let request = request.set.prop;

    let calendar = Calendar {
        id: cal_id.to_owned(),
        principal: principal.to_owned(),
        order: request.order.unwrap_or(0),
        displayname: request.displayname,
        timezone: request.calendar_timezone,
        color: request.calendar_color,
        description: request.calendar_description,
        deleted_at: None,
        synctoken: 0,
    };

    match store.read().await.get_calendar(&principal, &cal_id).await {
        Err(rustical_store::Error::NotFound) => {
            // No conflict, no worries
        }
        Ok(_) => {
            // oh no, there's a conflict
            return Ok(HttpResponse::Conflict().body("A calendar already exists at this URI"));
        }
        Err(err) => {
            // some other error
            return Err(err.into());
        }
    }

    match store.write().await.insert_calendar(calendar).await {
        // The spec says we should return a mkcalendar-response but I don't know what goes into it.
        // However, it works without one but breaks on iPadOS when using an empty one :)
        Ok(()) => Ok(HttpResponse::Created()
            .insert_header(("Cache-Control", "no-cache"))
            .body("")),
        Err(err) => {
            dbg!(err.to_string());
            Err(err.into())
        }
    }
}
