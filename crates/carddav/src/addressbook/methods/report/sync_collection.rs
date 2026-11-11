use std::str::FromStr;

use crate::{
    Error,
    address_object::{
        AddressObjectPropWrapper, AddressObjectPropWrapperName, resource::AddressObjectResource,
    },
};
use http::{StatusCode, Uri};
use rustical_dav::{
    resource::{PrincipalUri, Resource},
    rfc_3986_percent_encode,
    xml::{
        MultistatusElement, multistatus::ResponseElement, sync_collection::SyncCollectionRequest,
    },
};
use rustical_store::{
    AddressbookStore,
    access::Resolved,
    auth::Principal,
    shared,
    synctoken::{format_synctoken, parse_synctoken},
};

pub async fn handle_sync_collection<AS: AddressbookStore>(
    sync_collection: &SyncCollectionRequest<AddressObjectPropWrapperName>,
    path: &str,
    puri: &impl PrincipalUri,
    user: &Principal,
    principal: &str,
    resolved: &Resolved,
    addr_store: &AS,
) -> Result<MultistatusElement<AddressObjectPropWrapper, String>, Error> {
    // RFC 6578 section 3.2: an empty token asks for the initial sync, an unknown one is an error
    let sync_token = sync_collection.sync_token.trim();
    let initial_sync = sync_token.is_empty();
    let old_synctoken = if initial_sync {
        0
    } else {
        parse_synctoken(sync_token).ok_or(rustical_dav::Error::InvalidSyncToken)?
    };
    let (new_objects, mut deleted_objects, new_synctoken) =
        shared::address_sync_changes(addr_store, resolved, old_synctoken).await?;
    // RFC 6578 section 3.4: the initial sync reports no removed members
    if initial_sync {
        deleted_objects.clear();
    }

    let mut responses = Vec::new();
    for (object_id, object) in new_objects {
        let path = format!(
            "{}/{object_id}.vcf",
            path.trim_end_matches('/'),
            object_id = rfc_3986_percent_encode(&object_id)
        );
        responses.push(
            AddressObjectResource {
                object,
                object_id,
                principal: principal.to_owned(),
                resolved: resolved.clone(),
            }
            .propfind(&path, &sync_collection.prop, None, puri, user)?,
        );
    }

    for object_id in deleted_objects {
        let path = format!(
            "{}/{object_id}.vcf",
            path.trim_end_matches('/'),
            object_id = rfc_3986_percent_encode(&object_id)
        );
        responses.push(ResponseElement {
            href: Uri::from_str(&path).unwrap(),
            status: Some(StatusCode::NOT_FOUND),
            propstat: vec![],
        });
    }

    Ok(MultistatusElement {
        responses,
        sync_token: Some(format_synctoken(new_synctoken)),
        ..Default::default()
    })
}
