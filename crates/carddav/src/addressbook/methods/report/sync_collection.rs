use crate::{
    address_object::resource::{AddressObjectPropWrapper, AddressObjectResource},
    Error,
};
use actix_web::{http::StatusCode, HttpRequest};
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
    AddressbookStore,
};

pub async fn handle_sync_collection<AS: AddressbookStore>(
    sync_collection: SyncCollectionRequest,
    req: HttpRequest,
    user: &User,
    principal: &str,
    addressbook_id: &str,
    addr_store: &AS,
) -> Result<MultistatusElement<AddressObjectPropWrapper, String>, Error> {
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
    let (new_objects, deleted_objects, new_synctoken) = addr_store
        .sync_changes(principal, addressbook_id, old_synctoken)
        .await?;

    let mut responses = Vec::new();
    for object in new_objects {
        let path = format!("{}/{}", req.path().trim_end_matches('/'), object.get_id());
        responses.push(
            AddressObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(&path, &props, user)?,
        );
    }

    for object_id in deleted_objects {
        let path = format!("{}/{}", req.path().trim_end_matches('/'), object_id);
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
