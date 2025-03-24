use axum::http::StatusCode;
use rustical_dav::{
    resource::Resource,
    xml::{
        multistatus::ResponseElement, sync_collection::SyncCollectionRequest, MultistatusElement,
        PropElement, PropfindType,
    },
};
use rustical_store::{
    auth::User,
    synctoken::{format_synctoken, parse_synctoken},
    CalendarStore,
};

use crate::{
    calendar_object::resource::{CalendarObjectPropWrapper, CalendarObjectResource},
    Error,
};

pub async fn handle_sync_collection<C: CalendarStore>(
    prefix: &str,
    sync_collection: SyncCollectionRequest,
    user: &User,
    principal: &str,
    cal_id: &str,
    cal_store: &C,
) -> Result<MultistatusElement<CalendarObjectPropWrapper, String>, Error> {
    let props = match sync_collection.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            vec!["propname".to_owned()]
        }
        PropfindType::Prop(PropElement(prop_tags)) => {
            prop_tags.into_iter().map(|propname| propname.0).collect()
        }
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let old_synctoken = parse_synctoken(&sync_collection.sync_token).unwrap_or(0);
    let (new_objects, deleted_objects, new_synctoken) = cal_store
        .sync_changes(principal, cal_id, old_synctoken)
        .await?;

    let mut responses = Vec::new();
    for object in new_objects {
        let path = format!("{}/{}", prefix, object.get_id());
        responses.push(
            CalendarObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(prefix, &path, &props, user)?,
        );
    }

    for object_id in deleted_objects {
        let path = format!("{}/{}", prefix, object_id);
        responses.push(ResponseElement {
            href: path,
            status: Some(StatusCode::NOT_FOUND),
            ..Default::default()
        });
    }

    Ok(MultistatusElement {
        responses,
        sync_token: Some(format_synctoken(new_synctoken)),
        ..Default::default()
    })
}
