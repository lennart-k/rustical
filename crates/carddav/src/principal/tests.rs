use rustical_dav::resource::Resource;
use rustical_store::auth::Principal;
use rustical_xml::XmlSerializeRoot;

use crate::{CardDavPrincipalUri, principal::PrincipalResource};

#[test]
fn test_propfind() {
    let propfind = PrincipalResource::parse_propfind(
        r#"<?xml version="1.0" encoding="UTF-8"?><propfind xmlns="DAV:"><allprop/></propfind>"#,
    )
    .unwrap();

    insta::assert_debug_snapshot!(propfind);

    let principal = Principal {
        id: "user".to_string(),
        displayname: None,
        principal_type: rustical_store::auth::PrincipalType::Individual,
        password: None,
        memberships: vec!["group".to_string()],
    };

    let resource = PrincipalResource {
        principal: principal.clone(),
        members: vec![],
    };

    let response = resource
        .propfind(
            &format!("/carddav/principal/{}", principal.id),
            &propfind.prop,
            propfind.include.as_ref(),
            &CardDavPrincipalUri("/carddav"),
            &principal,
        )
        .unwrap();

    insta::assert_debug_snapshot!(response);
    insta::assert_snapshot!(response.serialize_to_string().unwrap());
}
