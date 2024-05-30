use crate::CalDavContext;
use crate::Error;
use actix_web::web::{Data, Path};
use actix_web::HttpResponse;
use anyhow::Result;
use rustical_auth::{AuthInfoExtractor, CheckAuthentication};
use rustical_dav::xml::tag_list::TagList;
use rustical_store::calendar::{Calendar, CalendarStore};
use serde::{Deserialize, Serialize};

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
    #[serde(flatten)]
    comp: Vec<CalendarComponentElement>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct MkcolCalendarProp {
    resourcetype: Resourcetype,
    displayname: Option<String>,
    calendar_description: Option<String>,
    calendar_color: Option<String>,
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

// TODO: Not sure yet what to send back :)
#[derive(Serialize, Clone, Debug)]
#[serde(rename = "mkcalendar-response")]
struct MkcalendarResponse;

pub async fn route_mkcol_calendar<A: CheckAuthentication, C: CalendarStore + ?Sized>(
    path: Path<(String, String)>,
    body: String,
    auth: AuthInfoExtractor<A>,
    context: Data<CalDavContext<C>>,
) -> Result<HttpResponse, Error> {
    let (principal, cid) = path.into_inner();
    if principal != auth.inner.user_id {
        return Err(Error::Unauthorized);
    }

    let request: MkcalendarRequest = quick_xml::de::from_str(&body).map_err(|e| {
        dbg!(e.to_string());
        Error::BadRequest
    })?;
    let request = request.set.prop;

    let calendar = Calendar {
        id: cid.to_owned(),
        owner: principal,
        name: request.displayname,
        timezone: request.calendar_timezone,
        color: request.calendar_color,
        description: request.calendar_description,
    };

    match context
        .store
        .write()
        .await
        .insert_calendar(cid, calendar)
        .await
    {
        Ok(()) => {
            let response = quick_xml::se::to_string(&MkcalendarResponse).unwrap();
            Ok(HttpResponse::Created().body(response))
        }
        Err(_err) => Ok(HttpResponse::InternalServerError().body("")),
    }
}
