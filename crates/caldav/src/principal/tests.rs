use std::sync::Arc;

use crate::principal::PrincipalResourceService;
use rstest::rstest;
use rustical_dav::resource::ResourceService;
use rustical_store_sqlite::{
    SqliteStore,
    calendar_store::SqliteCalendarStore,
    principal_store::SqlitePrincipalStore,
    tests::{get_test_calendar_store, get_test_principal_store, get_test_subscription_store},
};

#[rstest]
#[tokio::test]
async fn test_principal_resource(
    #[from(get_test_calendar_store)]
    #[future]
    cal_store: SqliteCalendarStore,
    #[from(get_test_principal_store)]
    #[future]
    auth_provider: SqlitePrincipalStore,
    #[from(get_test_subscription_store)]
    #[future]
    sub_store: SqliteStore,
) {
    let service = PrincipalResourceService {
        cal_store: Arc::new(cal_store.await),
        sub_store: Arc::new(sub_store.await),
        auth_provider: Arc::new(auth_provider.await),
    };

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
async fn test_propfind() {}
