use crate::{
    CalDavPrincipalUri,
    principal::{PrincipalResource, PrincipalResourceService},
};
use rstest::rstest;
use rustical_dav::resource::{Resource, ResourceService};
use rustical_store::auth::{Principal, PrincipalType::Individual};
use rustical_store_sqlite::tests::{TestStoreContext, test_store_context};
use rustical_xml::XmlSerializeRoot;
use std::sync::Arc;

#[rstest]
#[tokio::test]
async fn test_principal_resource(
    #[future]
    #[from(test_store_context)]
    context: TestStoreContext,
) {
    let TestStoreContext {
        cal_store,
        sub_store,
        principal_store: auth_provider,
        ..
    } = context.await;
    let service = PrincipalResourceService {
        cal_store: Arc::new(cal_store),
        sub_store: Arc::new(sub_store),
        auth_provider: Arc::new(auth_provider),
        simplified_home_set: false,
        config: Default::default(),
    };

    // We don't have any calendars here
    assert!(
        service
            .get_members(&("user".to_owned(),))
            .await
            .unwrap()
            .is_empty()
    );

    assert!(matches!(
        service
            .get_resource(&("invalid-user".to_owned(),), true)
            .await,
        Err(crate::Error::NotFound)
    ));

    let _principal_resource = service
        .get_resource(&("user".to_owned(),), true)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_propfind() {
    let propfind = PrincipalResource::parse_propfind(
        r#"<?xml version="1.0" encoding="UTF-8"?><propfind xmlns="DAV:"><allprop/></propfind>"#,
    )
    .unwrap();

    insta::assert_debug_snapshot!(propfind);

    let principal = Principal {
        id: "user".to_string(),
        displayname: None,
        principal_type: Individual,
        password: None,
        memberships: vec!["group".to_string()],
    };

    let resource = PrincipalResource {
        principal: principal.clone(),
        members: vec![],
        simplified_home_set: false,
    };

    let response = resource
        .propfind(
            &format!("/caldav/principal/{}", principal.id),
            &propfind.prop,
            propfind.include.as_ref(),
            &CalDavPrincipalUri("/caldav"),
            &principal,
        )
        .unwrap();

    insta::assert_debug_snapshot!(response);
    insta::assert_snapshot!(response.serialize_to_string().unwrap());
}
