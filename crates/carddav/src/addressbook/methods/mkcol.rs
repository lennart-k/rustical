use crate::{Error, addressbook::AddressbookResourceService};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use rustical_store::{Addressbook, AddressbookStore, SubscriptionStore, auth::Principal};
use rustical_xml::{XmlDeserialize, XmlDocument, XmlRootTag};
use tracing::instrument;

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct Resourcetype {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    addressbook: Option<()>,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    collection: Option<()>,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct MkcolAddressbookProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    resourcetype: Option<Resourcetype>,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    displayname: Option<String>,
    #[xml(rename = "addressbook-description")]
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    description: Option<String>,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct PropElement<T: XmlDeserialize> {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    prop: T,
}

#[derive(XmlDeserialize, XmlRootTag, Clone, Debug, PartialEq)]
#[xml(root = "mkcol")]
#[xml(ns = "rustical_dav::namespace::NS_DAV")]
struct MkcolRequest {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    set: PropElement<MkcolAddressbookProp>,
}

#[instrument(skip(addr_store))]
pub async fn route_mkcol<AS: AddressbookStore, S: SubscriptionStore>(
    Path((principal, addressbook_id)): Path<(String, String)>,
    user: Principal,
    State(AddressbookResourceService { addr_store, .. }): State<AddressbookResourceService<AS, S>>,
    body: String,
) -> Result<Response, Error> {
    if !user.is_principal(&principal) {
        return Err(Error::Unauthorized);
    }

    let mut request = MkcolRequest::parse_str(&body)?.set.prop;
    if let Some("") = request.displayname.as_deref() {
        request.displayname = None
    }

    let addressbook = Addressbook {
        id: addressbook_id.to_owned(),
        principal: principal.to_owned(),
        displayname: request.displayname,
        description: request.description,
        deleted_at: None,
        synctoken: 0,
        push_topic: uuid::Uuid::new_v4().to_string(),
    };

    match addr_store
        .get_addressbook(&principal, &addressbook_id, true)
        .await
    {
        Err(rustical_store::Error::NotFound) => {
            // No conflict, no worries
        }
        Ok(_) => {
            // oh no, there's a conflict
            return Ok((
                StatusCode::CONFLICT,
                "An addressbook already exists at this URI",
            )
                .into_response());
        }
        Err(err) => {
            // some other error
            return Err(err.into());
        }
    }

    addr_store.insert_addressbook(addressbook).await?;
    Ok(StatusCode::CREATED.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_mkcol() {
        let mkcol_request = MkcolRequest::parse_str(r#"
            <?xml version='1.0' encoding='UTF-8' ?>
            <mkcol xmlns="DAV:" xmlns:CAL="urn:ietf:params:xml:ns:caldav" xmlns:CARD="urn:ietf:params:xml:ns:carddav">
                <set>
                    <prop>
                        <resourcetype>
                            <collection />
                            <CARD:addressbook />
                        </resourcetype>
                        <displayname>whoops</displayname>
                        <CARD:addressbook-description>okay</CARD:addressbook-description>
                    </prop>
                </set>
            </mkcol>
        "#).unwrap();
        assert_eq!(
            mkcol_request,
            MkcolRequest {
                set: PropElement {
                    prop: MkcolAddressbookProp {
                        resourcetype: Some(Resourcetype {
                            addressbook: Some(()),
                            collection: Some(())
                        }),
                        displayname: Some("whoops".to_owned()),
                        description: Some("okay".to_owned())
                    }
                }
            }
        )
    }
}
