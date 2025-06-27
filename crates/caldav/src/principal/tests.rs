use crate::principal::PrincipalResourceService;
use rustical_dav::resource::ResourceService;
use rustical_store::auth::{AuthenticationProvider, Principal, PrincipalType};
use rustical_store_sqlite::tests::get_test_stores;

#[tokio::test]
async fn test_principal_resource() {
    let (_, cal_store, sub_store, auth_provider, _) = get_test_stores().await;
    let service = PrincipalResourceService {
        cal_store,
        sub_store,
        auth_provider: auth_provider.clone(),
    };

    auth_provider
        .insert_principal(
            Principal {
                id: "user".to_owned(),
                displayname: None,
                memberships: vec![],
                password: None,
                principal_type: PrincipalType::Individual,
            },
            true,
        )
        .await
        .unwrap();

    assert!(matches!(
        service.get_resource(&("anonymous".to_owned(),), true).await,
        Err(crate::Error::NotFound)
    ));

    let _principal_resource = service
        .get_resource(&("user".to_owned(),), true)
        .await
        .unwrap();
}

