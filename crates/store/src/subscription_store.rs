use crate::Error;
use async_trait::async_trait;
use chrono::NaiveDateTime;

pub struct Subscription {
    pub id: String,
    pub topic: String,
    pub expiration: NaiveDateTime,
    pub push_resource: String,
}

#[async_trait]
pub trait SubscriptionStore: Send + Sync + 'static {
    async fn get_subscriptions(&self, topic: &str) -> Result<Vec<Subscription>, Error>;
    async fn get_subscription(&self, id: &str) -> Result<Subscription, Error>;
    /// Returns whether a subscription under the id already existed
    async fn upsert_subscription(&self, sub: Subscription) -> Result<bool, Error>;
    async fn delete_subscription(&self, id: &str) -> Result<(), Error>;
}
