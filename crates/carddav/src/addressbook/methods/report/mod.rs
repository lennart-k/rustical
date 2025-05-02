use crate::Error;
use actix_web::{
    HttpRequest, Responder,
    web::{Data, Path},
};
use addressbook_multiget::{AddressbookMultigetRequest, handle_addressbook_multiget};
use rustical_dav::xml::{PropElement, PropfindType, sync_collection::SyncCollectionRequest};
use rustical_store::{AddressbookStore, auth::User};
use rustical_xml::{XmlDeserialize, XmlDocument};
use sync_collection::handle_sync_collection;
use tracing::instrument;

mod addressbook_multiget;
mod sync_collection;

#[derive(XmlDeserialize, XmlDocument, Clone, Debug, PartialEq)]
pub(crate) enum ReportRequest {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookMultiget(AddressbookMultigetRequest),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SyncCollection(SyncCollectionRequest),
}

impl ReportRequest {
    fn props(&self) -> Vec<&str> {
        let prop_element = match self {
            ReportRequest::AddressbookMultiget(AddressbookMultigetRequest { prop, .. }) => prop,
            ReportRequest::SyncCollection(SyncCollectionRequest { prop, .. }) => prop,
        };

        match prop_element {
            PropfindType::Allprop => {
                vec!["allprop"]
            }
            PropfindType::Propname => {
                vec!["propname"]
            }
            PropfindType::Prop(PropElement(prop_tags)) => prop_tags
                .iter()
                .map(|propname| propname.0.as_str())
                .collect(),
        }
    }
}

#[instrument(skip(req, addr_store))]
pub async fn route_report_addressbook<AS: AddressbookStore>(
    path: Path<(String, String)>,
    body: String,
    user: User,
    req: HttpRequest,
    addr_store: Data<AS>,
) -> Result<impl Responder, Error> {
    let (principal, addressbook_id) = path.into_inner();
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let request = ReportRequest::parse_str(&body)?;
    let props = request.props();

    Ok(match &request {
        ReportRequest::AddressbookMultiget(addr_multiget) => {
            handle_addressbook_multiget(
                addr_multiget,
                &props,
                req,
                &user,
                &principal,
                &addressbook_id,
                addr_store.as_ref(),
            )
            .await?
        }
        ReportRequest::SyncCollection(sync_collection) => {
            handle_sync_collection(
                sync_collection,
                &props,
                req,
                &user,
                &principal,
                &addressbook_id,
                addr_store.as_ref(),
            )
            .await?
        }
    })
}

#[cfg(test)]
mod tests {
    use rustical_dav::xml::{PropElement, Propname, sync_collection::SyncLevel};

    use super::*;

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
                sync_token: "".to_owned(),
                sync_level: SyncLevel::One,
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(vec![Propname(
                    "getetag".to_owned()
                )])),
                limit: None
            })
        )
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
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(vec![
                    Propname("getetag".to_owned()),
                    Propname("address-data".to_owned())
                ])),
                href: vec![
                    "/carddav/user/user/6f787542-5256-401a-8db97003260da/ae7a998fdfd1d84a20391168962c62b".to_owned()
                ]
            })
        )
    }
}
