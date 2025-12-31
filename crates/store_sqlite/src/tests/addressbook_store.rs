#[cfg(test)]
mod tests {
    use crate::tests::{TestStoreContext, test_store_context};
    use rstest::rstest;
    use rustical_store::{Addressbook, AddressbookStore};

    #[rstest]
    #[tokio::test]
    async fn test_addressbook_store(
        #[from(test_store_context)]
        #[future]
        context: TestStoreContext,
    ) {
        let addr_store = context.await.addr_store;

        let cal = Addressbook {
            id: "addr".to_string(),
            principal: "fake-user".to_string(),
            displayname: None,
            description: None,
            deleted_at: None,
            synctoken: 0,
            push_topic: "alskdj".to_string(),
        };

        assert!(
            addr_store.insert_addressbook(cal).await.is_err(),
            "This should fail due to the user not existing "
        );

        let addr = Addressbook {
            id: "addr".to_string(),
            principal: "user".to_string(),
            displayname: None,
            description: None,
            deleted_at: None,
            synctoken: 0,
            push_topic: "alskdj".to_string(),
        };

        addr_store.insert_addressbook(addr.clone()).await.unwrap();

        assert_eq!(
            addr_store
                .get_addressbook("user", "addr", false)
                .await
                .unwrap(),
            addr
        );

        addr_store
            .delete_addressbook("user", "addr", true)
            .await
            .unwrap();

        let Err(err) = addr_store.get_addressbook("user", "addr", false).await else {
            panic!()
        };
        assert!(err.is_not_found());

        addr_store
            .get_addressbook("user", "addr", true)
            .await
            .unwrap();

        addr_store
            .restore_addressbook("user", "addr")
            .await
            .unwrap();

        addr_store
            .delete_addressbook("user", "addr", false)
            .await
            .unwrap();

        let Err(err) = addr_store.get_addressbook("user", "addr", true).await else {
            panic!()
        };
        assert!(err.is_not_found());
    }
}
