use crate::{CardDavPrincipalUri, addressbook::resource::AddressbookResource};
use rustical_dav::resource::Resource;
use rustical_dav_push::VapidPublicKeyB64;
use rustical_store::{Addressbook, auth::Principal};
use rustical_xml::XmlSerializeRoot;

#[test]
fn test_propfind() {
    let propfind = AddressbookResource::parse_propfind(
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

    let addressbook = Addressbook {
        id: "yeet".to_string(),
        principal: "user".to_string(),
        displayname: None,
        description: None,
        deleted_at: None,
        synctoken: 0,
        push_topic: "asdasd".to_string(),
    };

    let vapid_pubkey = VapidPublicKeyB64(
        "BPtEpPcF_eXYV3KXYOYRJvnrvdhNVFNfHzvV5GgTQQ_oXDwm-AKFJw_DLnc_d4reJc7SFiuFtYRUkCisT_4TRz"
            .to_string(),
    );

    let resource = AddressbookResource {
        addressbook: addressbook.clone(),
        vapid_pubkey,
        advertise_vcard4: true,
    };
    let response = resource
        .propfind(
            &format!(
                "/carddav/principal/{}/{}",
                addressbook.principal, addressbook.id
            ),
            &propfind.prop,
            propfind.include.as_ref(),
            &CardDavPrincipalUri("/carddav"),
            &principal,
        )
        .unwrap();

    insta::assert_debug_snapshot!(response);
    insta::assert_snapshot!(response.serialize_to_string().unwrap());
}

#[rstest::rstest]
#[case(true, true)]
#[case(false, false)]
fn test_supported_address_data_advertise_vcard4(
    #[case] advertise_vcard4: bool,
    #[case] expect_vcard4: bool,
) {
    let propfind = AddressbookResource::parse_propfind(
        r#"<?xml version="1.0" encoding="UTF-8"?>
        <propfind xmlns="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
            <prop><C:supported-address-data/></prop>
        </propfind>"#,
    )
    .unwrap();

    let principal = Principal {
        id: "user".to_string(),
        displayname: None,
        principal_type: rustical_store::auth::PrincipalType::Individual,
        password: None,
        memberships: vec![],
    };
    let addressbook = Addressbook {
        id: "yeet".to_string(),
        principal: "user".to_string(),
        displayname: None,
        description: None,
        deleted_at: None,
        synctoken: 0,
        push_topic: "asdasd".to_string(),
    };
    let resource = AddressbookResource {
        addressbook: addressbook.clone(),
        vapid_pubkey: VapidPublicKeyB64("test".to_string()),
        advertise_vcard4,
    };

    let xml = resource
        .propfind(
            &format!(
                "/carddav/principal/{}/{}",
                addressbook.principal, addressbook.id
            ),
            &propfind.prop,
            propfind.include.as_ref(),
            &CardDavPrincipalUri("/carddav"),
            &principal,
        )
        .unwrap()
        .serialize_to_string()
        .unwrap();

    assert!(
        xml.contains(r#"version="3.0""#),
        "vCard 3.0 must always be advertised: {xml}"
    );
    assert_eq!(
        xml.contains(r#"version="4.0""#),
        expect_vcard4,
        "advertise_vcard4={advertise_vcard4}: {xml}"
    );
}
