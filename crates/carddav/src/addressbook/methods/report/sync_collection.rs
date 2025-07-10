use crate::{
    Error,
    address_object::{
        AddressObjectPropWrapper, AddressObjectPropWrapperName, resource::AddressObjectResource,
    },
};
use http::StatusCode;
use rustical_dav::{
    resource::{PrincipalUri, Resource},
    xml::{
        MultistatusElement, multistatus::ResponseElement, sync_collection::SyncCollectionRequest,
    },
};
use rustical_store::{
    AddressbookStore,
    auth::Principal,
    synctoken::{format_synctoken, parse_synctoken},
};

pub async fn handle_sync_collection<AS: AddressbookStore>(
    sync_collection: &SyncCollectionRequest<AddressObjectPropWrapperName>,
    path: &str,
    puri: &impl PrincipalUri,
    user: &Principal,
    principal: &str,
    addressbook_id: &str,
    addr_store: &AS,
) -> Result<MultistatusElement<AddressObjectPropWrapper, String>, Error> {
    let old_synctoken = parse_synctoken(&sync_collection.sync_token).unwrap_or(0);
    let (new_objects, deleted_objects, new_synctoken) = addr_store
        .sync_changes(principal, addressbook_id, old_synctoken)
        .await?;

    let mut responses = Vec::new();
    for object in new_objects {
        let path = format!("{}/{}.vcf", path.trim_end_matches('/'), object.get_id());
        responses.push(
            AddressObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(&path, &sync_collection.prop, None, puri, user)?,
        );
    }

    for object_id in deleted_objects {
        let path = format!("{}/{}.vcf", path.trim_end_matches('/'), object_id);
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
