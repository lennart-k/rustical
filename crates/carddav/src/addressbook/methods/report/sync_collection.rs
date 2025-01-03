use crate::{
    address_object::resource::{AddressObjectProp, AddressObjectResource},
    Error,
};
use actix_web::{http::StatusCode, HttpRequest};
use rustical_dav::{
    resource::{CommonPropertiesProp, EitherProp, Resource},
    xml::{multistatus::ResponseElement, MultistatusElement},
    xml::{PropElement, PropfindType},
};
use rustical_store::{
    auth::User,
    synctoken::{format_synctoken, parse_synctoken},
    AddressbookStore,
};
use rustical_xml::{Value, XmlDeserialize};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SyncLevel {
    One,
    Infinity,
}

impl Value for SyncLevel {
    fn deserialize(val: &str) -> Result<Self, rustical_xml::XmlDeError> {
        Ok(match val {
            "1" => Self::One,
            "Infinity" => Self::Infinity,
            // TODO: proper error
            _ => return Err(rustical_xml::XmlDeError::UnknownError),
        })
    }
    fn serialize(&self) -> String {
        match self {
            SyncLevel::One => "1",
            SyncLevel::Infinity => "Infinity",
        }
        .to_owned()
    }
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
// <!ELEMENT sync-collection (sync-token, sync-level, limit?, prop)>
//    <!-- DAV:limit defined in RFC 5323, Section 5.17 -->
//    <!-- DAV:prop defined in RFC 4918, Section 14.18 -->
pub(crate) struct SyncCollectionRequest {
    pub(crate) sync_token: String,
    pub(crate) sync_level: SyncLevel,
    #[xml(ty = "untagged")]
    pub prop: PropfindType,
    pub(crate) limit: Option<u64>,
}

pub async fn handle_sync_collection<AS: AddressbookStore + ?Sized>(
    sync_collection: SyncCollectionRequest,
    req: HttpRequest,
    user: &User,
    principal: &str,
    addressbook_id: &str,
    addr_store: &AS,
) -> Result<MultistatusElement<EitherProp<AddressObjectProp, CommonPropertiesProp>, String>, Error>
{
    let props = match sync_collection.prop {
        PropfindType::Allprop => {
            vec!["allprop".to_owned()]
        }
        PropfindType::Propname => {
            vec!["propname".to_owned()]
        }
        PropfindType::Prop(PropElement { prop: prop_tags }) => prop_tags
            .into_iter()
            .map(|propname| propname.name)
            .collect(),
    };
    let props: Vec<&str> = props.iter().map(String::as_str).collect();

    let old_synctoken = parse_synctoken(&sync_collection.sync_token).unwrap_or(0);
    let (new_objects, deleted_objects, new_synctoken) = addr_store
        .sync_changes(principal, addressbook_id, old_synctoken)
        .await?;

    let mut responses = Vec::new();
    for object in new_objects {
        let path = AddressObjectResource::get_url(
            req.resource_map(),
            vec![principal, addressbook_id, &object.get_id()],
        )
        .unwrap();
        responses.push(
            AddressObjectResource {
                object,
                principal: principal.to_owned(),
            }
            .propfind(&path, &props, user, req.resource_map())?,
        );
    }

    for object_id in deleted_objects {
        let path = AddressObjectResource::get_url(
            req.resource_map(),
            vec![principal, addressbook_id, &object_id],
        )
        .unwrap();
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
