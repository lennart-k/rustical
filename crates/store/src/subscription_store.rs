use crate::Error;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone};

pub struct Subscription {
    pub id: String,
    pub topic: String,
    // Naive because sqlite has no concept of timezones
    // In reality, this is UTC
    pub expiration: NaiveDateTime,
    pub push_resource: String,
    pub public_key: String,
    pub public_key_type: String,
    pub auth_secret: String,
}

impl Subscription {
    #[must_use]
    pub fn is_expired(&self, now: &DateTime<impl TimeZone>) -> bool {
        self.expiration < now.naive_utc()
    }
}

#[async_trait]
pub trait SubscriptionStore: Send + Sync + 'static {
    async fn get_subscriptions(&self, topic: &str) -> Result<Vec<Subscription>, Error>;
    async fn get_subscription(&self, id: &str) -> Result<Subscription, Error>;
    /// Returns whether a subscription under the id already existed
    async fn upsert_subscription(&self, sub: Subscription) -> Result<bool, Error>;
    async fn delete_subscription(&self, id: &str) -> Result<(), Error>;
}
