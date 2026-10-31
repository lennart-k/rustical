use std::str::FromStr;

use crate::{
    CardDavPrincipalUri, Error,
    address_object::{
        AddressObjectPropWrapper, AddressObjectPropWrapperName, resource::AddressObjectResource,
    },
    addressbook::{
        AddressbookResourceService,
        methods::report::addressbook_query::{
            AddressbookQueryRequest, get_objects_addressbook_query,
        },
    },
};
use addressbook_multiget::{AddressbookMultigetRequest, handle_addressbook_multiget};
use axum::{
    Extension,
    extract::{OriginalUri, Path, State},
    response::IntoResponse,
};
use http::{StatusCode, Uri};
use rustical_dav::{
    resource::{PrincipalUri, Resource},
    rfc_3986_percent_encode,
    xml::{
        MultistatusElement, PropfindType, multistatus::ResponseElement,
        sync_collection::SyncCollectionRequest,
    },
};
use rustical_dav_push::DavPushStore;
use rustical_ical::AddressObject;
use rustical_store::{
    AddressbookStore,
    access::{self, Need, Resolved},
    auth::Principal,
};
use rustical_xml::{XmlDeserialize, XmlDocument};
use sync_collection::handle_sync_collection;
use tracing::instrument;

mod addressbook_multiget;
mod addressbook_query;
mod sync_collection;

#[derive(XmlDeserialize, XmlDocument, Clone, Debug, PartialEq)]
pub(crate) enum ReportRequest {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookMultiget(AddressbookMultigetRequest),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookQuery(AddressbookQueryRequest),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SyncCollection(SyncCollectionRequest<AddressObjectPropWrapperName>),
}

impl ReportRequest {
    const fn props(&self) -> &PropfindType<AddressObjectPropWrapperName> {
        match self {
            Self::AddressbookMultiget(AddressbookMultigetRequest { prop, .. })
            | Self::SyncCollection(SyncCollectionRequest { prop, .. })
            | Self::AddressbookQuery(AddressbookQueryRequest { prop, .. }) => prop,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn objects_response(
    objects: Vec<(String, AddressObject)>,
    not_found: Vec<String>,
    path: &str,
    principal: &str,
    resolved: &Resolved,
    puri: &impl PrincipalUri,
    user: &Principal,
    prop: &PropfindType<AddressObjectPropWrapperName>,
) -> Result<MultistatusElement<AddressObjectPropWrapper, String>, Error> {
    let mut responses = Vec::new();
    for (object_id, object) in objects {
        let path = format!(
            "{}/{object_id}.vcf",
            path,
            object_id = rfc_3986_percent_encode(&object_id)
        );
        responses.push(
            AddressObjectResource {
                object,
                object_id,
                principal: principal.to_owned(),
                resolved: resolved.clone(),
            }
            .propfind(&path, prop, None, puri, user)?,
        );
    }

    let not_found_responses = not_found
        .into_iter()
        .map(|path| ResponseElement {
            href: Uri::from_str(&path).unwrap(),
            status: Some(StatusCode::NOT_FOUND),
            propstat: vec![],
        })
        .collect();

    Ok(MultistatusElement {
        responses,
        member_responses: not_found_responses,
        ..Default::default()
    })
}

#[instrument(skip(service))]
pub async fn route_report_addressbook<AS: AddressbookStore, DP: DavPushStore>(
    Path((principal, addressbook_id)): Path<(String, String)>,
    user: Principal,
    OriginalUri(uri): OriginalUri,
    Extension(puri): Extension<CardDavPrincipalUri>,
    State(service): State<AddressbookResourceService<AS, DP>>,
    body: String,
) -> Result<impl IntoResponse, Error> {
    access::check_home(&user, &principal)?;
    let addressbook = service.resource(&principal, &addressbook_id, false).await?;
    let resolved = &addressbook.resolved;
    resolved.check(Need::Read)?;
    let addr_store = service.addr_store.as_ref();

    let request = ReportRequest::parse_str(&body)?;

    Ok(match &request {
        ReportRequest::AddressbookMultiget(addr_multiget) => {
            handle_addressbook_multiget(
                addr_multiget,
                request.props(),
                &uri,
                &puri,
                &user,
                &principal,
                resolved,
                addr_store,
            )
            .await?
        }
        ReportRequest::SyncCollection(sync_collection) => {
            handle_sync_collection(
                sync_collection,
                uri.path(),
                &puri,
                &user,
                &principal,
                resolved,
                addr_store,
            )
            .await?
        }
        ReportRequest::AddressbookQuery(addr_query) => {
            let objects = get_objects_addressbook_query(
                addr_query,
                resolved.owner(),
                resolved.id(),
                addr_store,
            )
            .await?;
            objects_response(
                objects,
                vec![],
                uri.path(),
                &principal,
                resolved,
                &puri,
                &user,
                &addr_query.prop,
            )?
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        address_object::AddressObjectPropName,
        addressbook::methods::report::addressbook_query::{
            Allof, FilterElement, LimitElement, NresultsElement, PropFilterElement,
        },
    };
    use rustical_dav::xml::{PropElement, sync_collection::SyncLevel};

    #[test]
    fn test_xml_sync_collection() {
        let report_request = ReportRequest::parse_str(
            r#"
        <?xml version='1.0' encoding='UTF-8' ?>
        <sync-collection xmlns="DAV:">
            <sync-token />
            <sync-level>1</sync-level>
            <prop>
                <getetag />
            </prop>
        </sync-collection>"#,
        )
        .unwrap();
        assert_eq!(
            report_request,
            ReportRequest::SyncCollection(SyncCollectionRequest {
                sync_token: String::new(),
                sync_level: SyncLevel::One,
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(
                    vec![AddressObjectPropWrapperName::AddressObject(
                        AddressObjectPropName::Getetag
                    )],
                    vec![]
                )),
                limit: None
            })
        );
    }

    #[test]
    fn test_xml_addressbook_multiget() {
        let report_request = ReportRequest::parse_str(r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <addressbook-multiget xmlns="urn:ietf:params:xml:ns:carddav" xmlns:D="DAV:">
                <D:prop>
                    <D:getetag/>
                    <address-data/>
                </D:prop>
                <D:href>/carddav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b</D:href>
            </addressbook-multiget>
        "#).unwrap();

        assert_eq!(
            report_request,
            ReportRequest::AddressbookMultiget(AddressbookMultigetRequest {
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(
                    vec![
                        AddressObjectPropWrapperName::AddressObject(AddressObjectPropName::Getetag),
                        AddressObjectPropWrapperName::AddressObject(
                            AddressObjectPropName::AddressData
                        ),
                    ],
                    vec![]
                )),
                href: vec![Uri::from_static(
                    "/carddav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b"
                )]
            })
        );
    }

    #[test]
    fn test_xml_addressbook_query() {
        let report_request = ReportRequest::parse_str(
            r#"
            <?xml version="1.0" encoding="utf-8"?>
                <card:addressbook-query xmlns:card="urn:ietf:params:xml:ns:carddav" xmlns:d="DAV:">
                <d:prop>
                    <d:getetag/>
                </d:prop>
                <card:filter>
                    <card:prop-filter name="FN"/>
                </card:filter>
                <card:limit>
                    <card:nresults>100</card:nresults>
                </card:limit>
            </card:addressbook-query>
        "#,
        )
        .unwrap();

        assert_eq!(
            report_request,
            ReportRequest::AddressbookQuery(AddressbookQueryRequest {
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(
                    vec![AddressObjectPropWrapperName::AddressObject(
                        AddressObjectPropName::Getetag
                    ),],
                    vec![]
                )),
                filter: FilterElement {
                    test: Allof::default(),
                    prop_filter: vec![PropFilterElement {
                        name: "FN".to_owned(),
                        is_not_defined: None,
                        text_match: vec![],
                        param_filter: vec![],
                        test: Allof::default()
                    }],
                },
                limit: Some(LimitElement {
                    nresults: NresultsElement(100)
                })
            })
        );
    }
}
