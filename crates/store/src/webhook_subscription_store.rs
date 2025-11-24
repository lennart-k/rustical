use crate::Error;
use async_trait::async_trait;
use rustical_types::WebhookSubscription;

#[async_trait]
pub trait WebhookSubscriptionStore: Send + Sync + 'static {
    async fn get_subscriptions(&self, resource_type: &str, resource_id: &str) -> Result<Vec<WebhookSubscription>, Error>;
    async fn get_subscription(&self, id: &str) -> Result<WebhookSubscription, Error>;
    /// Returns whether a subscription under the id already existed
    async fn upsert_subscription(&self, sub: WebhookSubscription) -> Result<bool, Error>;
    async fn delete_subscription(&self, id: &str) -> Result<(), Error>;
}