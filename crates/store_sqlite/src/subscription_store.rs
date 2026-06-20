use crate::SqliteStore;
use async_trait::async_trait;
use rustical_dav_push::{
    Subscription, SubscriptionReadStore, SubscriptionStore, SubscriptionWriteStore, VapidKeyStore,
    VapidKeypair,
};
use rustical_store::Error;

impl rustical_dav_push::WithError for SqliteStore {
    type Error = crate::Error;
}

#[async_trait]
impl VapidKeyStore for SqliteStore {
    async fn get_vapid_keypair(&self) -> Result<VapidKeypair, Self::Error> {
        // TODO: implement
        Ok(VapidKeypair::generate_p256().map_err(rustical_dav_push::Error::VapidError)?)
    }
}

#[async_trait]
impl SubscriptionReadStore for SqliteStore {
    async fn get_subscriptions(&self, topic: &str) -> Result<Vec<Subscription>, Self::Error> {
        Ok(sqlx::query_as!(
            Subscription,
            r#"SELECT id, topic, expiration, push_resource, public_key, public_key_type, auth_secret
                FROM davpush_subscriptions
                WHERE (topic) = (?)"#,
            topic
        )
        .fetch_all(&self.db)
        .await?)
    }

    async fn get_subscription(&self, id: &str) -> Result<Subscription, Self::Error> {
        Ok(sqlx::query_as!(
            Subscription,
            r#"SELECT id, topic, expiration, push_resource, public_key, public_key_type, auth_secret
                FROM davpush_subscriptions
                WHERE (id) = (?)"#,
            id
        )
        .fetch_one(&self.db)
        .await?)
    }
}

#[async_trait]
impl SubscriptionWriteStore for SqliteStore {
    async fn upsert_subscription(&self, sub: Subscription) -> Result<bool, Self::Error> {
        let already_exists = match self.get_subscription(&sub.id).await {
            Ok(_) => true,
            Err(crate::Error::StoreError(Error::NotFound)) => false,
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
        ).execute(&self.db).await?;
        Ok(already_exists)
    }
    async fn delete_subscription(&self, id: &str) -> Result<(), Self::Error> {
        sqlx::query!(r#"DELETE FROM davpush_subscriptions WHERE id = ? "#, id)
            .execute(&self.db)
            .await?;
        Ok(())
    }
}

impl SubscriptionStore for SqliteStore {}
