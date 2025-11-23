use crate::SqliteStore;
use async_trait::async_trait;
use rustical_store::{Error, WebhookSubscriptionStore};
use rustical_types::WebhookSubscription;

#[async_trait]
impl WebhookSubscriptionStore for SqliteStore {
    async fn get_subscriptions(&self, resource_type: &str, resource_id: &str) -> Result<Vec<WebhookSubscription>, Error>{
        Ok(sqlx::query_as!(
            WebhookSubscription,
            r#"SELECT id, target_url, resource_type, resource_id, secret_key
                FROM webhook_subscriptions
                WHERE (resource_type = ? AND resource_id = ?)"#,
            resource_type,
            resource_id
        )
        .fetch_all(&self.db)
        .await
        .map_err(crate::Error::from)?)
    }

    async fn get_subscription(&self, id: &str) -> Result<WebhookSubscription, Error>{
        Ok(sqlx::query_as!(
            WebhookSubscription,
            r#"SELECT id, target_url, resource_type, resource_id, secret_key
                FROM webhook_subscriptions
                WHERE (id) = (?)"#,
            id
        )
        .fetch_one(&self.db)
        .await
        .map_err(crate::Error::from)?)
    }
    
    // Returns whether a subscription under the id already existed
    async fn upsert_subscription(&self, sub: WebhookSubscription) -> Result<bool, Error>{
        let already_exists = match self.get_subscription(&sub.id).await {
            Ok(_) => true,
            Err(Error::NotFound) => false,
            Err(err) => return Err(err),
        };
        sqlx::query!(
            r#"INSERT OR REPLACE INTO webhook_subscriptions (id, target_url, resource_type, resource_id, secret_key) VALUES (?, ?, ?, ?, ?)"#,
            sub.id,
            sub.target_url,
            sub.resource_type,
            sub.resource_id,
            sub.secret_key
        ).execute(&self.db).await.map_err(crate::Error::from)?;
        Ok(already_exists)
    }
    
    async fn delete_subscription(&self, id: &str) -> Result<(), Error>{
        sqlx::query!(
            r#"DELETE FROM webhook_subscriptions WHERE id = ?"#,
            id
        ).execute(&self.db).await.map_err(crate::Error::from)?;
        Ok(())
    }
}