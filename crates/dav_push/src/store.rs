use crate::{Subscription, VapidKeypair, VapidPublicKeyB64};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rustical_store::Error;

#[async_trait]
pub trait SubscriptionStore: Send + Sync + 'static {
    async fn get_subscriptions(&self, topic: &str) -> Result<Vec<Subscription>, Error>;
    async fn get_subscription(&self, id: &str) -> Result<Subscription, Error>;
    /// Returns whether a subscription under the id already existed
    async fn upsert_subscription(&self, sub: Subscription) -> Result<bool, Error>;
    async fn delete_subscription(&self, id: &str) -> Result<(), Error>;
    /// Deletes all subscriptions that expired before `before` (UTC).
    /// Returns the number of deleted subscriptions
    async fn delete_expired_subscriptions(&self, before: DateTime<Utc>) -> Result<u64, Error>;
}

#[async_trait]
pub trait VapidStore: Send + Sync + 'static {
    // Returns a VapidKeypair. Generates a new one if none exists yet
    async fn get_vapid_keypair(&self) -> Result<&VapidKeypair, Error>;
    // Returns the base64-encoded public key
    async fn get_vapid_pubkey_b64(&self) -> Result<&VapidPublicKeyB64, Error>;
}

pub trait DavPushStore: SubscriptionStore + VapidStore {}

impl<T: SubscriptionStore + VapidStore> DavPushStore for T {}
