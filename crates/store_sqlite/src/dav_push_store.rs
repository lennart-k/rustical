use std::sync::Arc;

use crate::BEGIN_IMMEDIATE;
use async_trait::async_trait;
use rustical_dav_push::{
    Subscription, SubscriptionStore, VapidKeypair, VapidPublicKeyB64, VapidStore,
};
use rustical_store::Error;
use sqlx::{Executor, Sqlite, SqlitePool};
use tokio::sync::OnceCell;

#[derive(Debug, Clone)]
pub struct SqliteDavPushStore {
    db: SqlitePool,
    _vapid_pubkey_cache: OnceCell<VapidPublicKeyB64>,
    _vapid_privkey_cache: OnceCell<Arc<VapidKeypair>>,
}

impl SqliteDavPushStore {
    #[must_use]
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            _vapid_pubkey_cache: OnceCell::new(),
            _vapid_privkey_cache: OnceCell::new(),
        }
    }
}

#[async_trait]
impl SubscriptionStore for SqliteDavPushStore {
    async fn get_subscriptions(&self, topic: &str) -> Result<Vec<Subscription>, Error> {
        Ok(sqlx::query_as!(
            Subscription,
            r#"SELECT id, topic, expiration, push_resource, public_key, public_key_type, auth_secret
                FROM davpush_subscriptions
                WHERE (topic) = (?)"#,
            topic
        )
        .fetch_all(&self.db)
        .await
        .map_err(crate::Error::from)?)
    }

    async fn get_subscription(&self, id: &str) -> Result<Subscription, Error> {
        Ok(sqlx::query_as!(
            Subscription,
            r#"SELECT id, topic, expiration, push_resource, public_key, public_key_type, auth_secret
                FROM davpush_subscriptions
                WHERE (id) = (?)"#,
            id
        )
        .fetch_one(&self.db)
        .await
        .map_err(crate::Error::from)?)
    }

    async fn upsert_subscription(&self, sub: Subscription) -> Result<bool, Error> {
        let already_exists = match self.get_subscription(&sub.id).await {
            Ok(_) => true,
            Err(Error::NotFound) => false,
            Err(err) => return Err(err),
        };
        sqlx::query!(
            r#"REPLACE INTO davpush_subscriptions (id, topic, expiration, push_resource, public_key, public_key_type, auth_secret) VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            sub.id,
            sub.topic,
            sub.expiration,
            sub.push_resource,
            sub.public_key,
            sub.public_key_type,
            sub.auth_secret
        ).execute(&self.db).await.map_err(crate::Error::from)?;
        Ok(already_exists)
    }
    async fn delete_subscription(&self, id: &str) -> Result<(), Error> {
        sqlx::query!(r#"DELETE FROM davpush_subscriptions WHERE id = ? "#, id)
            .execute(&self.db)
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }
}

impl SqliteDavPushStore {
    pub async fn initialise(&self) -> Result<(), Error> {
        self.get_vapid_keypair().await?;
        self.get_vapid_pubkey_b64().await?;
        Ok(())
    }

    async fn insert_vapid_key<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
        key: &VapidKeypair,
    ) -> Result<(), Error> {
        // TODO: proper error
        let pem = key.to_pem().map_err(|err| Error::Other(err.into()))?;
        sqlx::query!("UPDATE davpush_vapid_key SET vapid_key = ?", pem)
            .execute(executor)
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }

    pub(super) async fn fetch_vapid_key<'e, E: Executor<'e, Database = Sqlite>>(
        executor: E,
    ) -> Result<Option<VapidKeypair>, Error> {
        struct Row {
            vapid_key: Option<String>,
        }
        let Some(Row {
            vapid_key: Some(pem),
        }) = sqlx::query_as!(Row, "SELECT vapid_key FROM davpush_vapid_key")
            .fetch_optional(executor)
            .await
            .map_err(crate::Error::from)?
        else {
            return Ok(None);
        };

        Ok(Some(
            VapidKeypair::from_pem(&pem).map_err(|err| Error::Other(err.into()))?,
        ))
    }

    async fn get_or_generate_vapid_keypair(&self) -> Result<VapidKeypair, Error> {
        let mut tx = self
            .db
            .begin_with(BEGIN_IMMEDIATE)
            .await
            .map_err(crate::Error::from)?;

        if let Some(key) = Self::fetch_vapid_key(&mut *tx).await? {
            return Ok(key);
        }

        let key = VapidKeypair::generate_p256();
        Self::insert_vapid_key(&mut *tx, &key).await?;
        tx.commit().await.map_err(crate::Error::from)?;
        Ok(key)
    }
}

#[async_trait]
impl VapidStore for SqliteDavPushStore {
    async fn get_vapid_keypair(&self) -> Result<&VapidKeypair, Error> {
        let keypair = self
            ._vapid_privkey_cache
            .get_or_try_init(async || self.get_or_generate_vapid_keypair().await.map(Arc::new))
            .await?;

        Ok(keypair.as_ref())
    }

    async fn get_vapid_pubkey_b64(&self) -> Result<&VapidPublicKeyB64, Error> {
        let pubkey = self
            ._vapid_pubkey_cache
            .get_or_try_init(async || {
                let key = self.get_vapid_keypair().await?;
                Result::<_, Error>::Ok(key.public().encode_b64())
            })
            .await?;

        Ok(pubkey)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        SqliteDavPushStore,
        tests::{TestStoreContext, test_store_context},
    };
    use rstest::rstest;
    use rustical_dav_push::VapidStore;

    #[rstest]
    #[tokio::test]
    async fn test_get_keypair(
        #[from(test_store_context)]
        #[future]
        context: TestStoreContext,
    ) {
        let store = context.await.dav_push_store;

        let key1 = store.get_vapid_keypair().await.unwrap();
        let key2 = store.get_vapid_keypair().await.unwrap();
        // Check that fetching it twice will yield the same
        assert_eq!(
            key1.to_pem().unwrap(),
            key2.to_pem().unwrap(),
            "test that cache is utilised"
        );

        let key_stored = SqliteDavPushStore::fetch_vapid_key(&store.db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            key1.to_pem().unwrap(),
            key_stored.to_pem().unwrap(),
            "test that the same key is stored in the database"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_get_public_key(
        #[from(test_store_context)]
        #[future]
        context: TestStoreContext,
    ) {
        let store = context.await.dav_push_store;

        let key1 = store.get_vapid_keypair().await.unwrap();
        let pubkey1 = key1.public().encode_b64();
        let pubkey2 = store.get_vapid_pubkey_b64().await.unwrap();
        assert_eq!(
            &pubkey1, pubkey2,
            "test that method returns the correct public key"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_initialise(
        #[from(test_store_context)]
        #[future]
        context: TestStoreContext,
    ) {
        let store = context.await.dav_push_store;
        store.initialise().await.unwrap();

        let key_stored = SqliteDavPushStore::fetch_vapid_key(&store.db)
            .await
            .unwrap()
            .unwrap();
        let key2 = store.get_vapid_keypair().await.unwrap();
        assert_eq!(
            key_stored.to_pem().unwrap(),
            key2.to_pem().unwrap(),
            "test that initialisation works"
        );
    }
}
