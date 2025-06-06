use crate::SqliteStore;
use async_trait::async_trait;
use rustical_store::{Error, Subscription, SubscriptionStore};

#[async_trait]
impl SubscriptionStore for SqliteStore {
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
        sqlx::query!(
            r#"INSERT OR REPLACE INTO davpush_subscriptions (id, topic, expiration, push_resource, public_key, public_key_type, auth_secret) VALUES (?, ?, ?, ?, ?, ?, ?)"#,
            sub.id,
            sub.topic,
            sub.expiration,
            sub.push_resource,
            sub.public_key,
            sub.public_key_type,
            sub.auth_secret
        ).execute(&self.db).await.map_err(crate::Error::from)?;
        // TODO: Correctly return whether a subscription already existed
        Ok(false)
    }
    async fn delete_subscription(&self, id: &str) -> Result<(), Error> {
        sqlx::query!(r#"DELETE FROM davpush_subscriptions WHERE id = ? "#, id)
            .execute(&self.db)
            .await
            .map_err(crate::Error::from)?;
        Ok(())
    }
}
