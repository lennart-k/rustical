use std::fmt::Debug;

use crate::vapid::{VapidError, VapidKeypair, VapidPublicKey};
use async_trait::async_trait;
use axum::response::IntoResponse;
use chrono::{DateTime, NaiveDateTime, TimeZone};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    VapidError(#[from] VapidError),
}

pub trait WithError {
    type Error: std::fmt::Display + From<Error> + IntoResponse;
}

#[async_trait]
/// Store to provide a VAPID key
pub trait VapidKeyStore: WithError + Send + Sync + 'static {
    // Returns a VAPID keypair and generates one if necessary
    async fn get_vapid_keypair(&self) -> Result<VapidKeypair, Self::Error>;
    async fn get_vapid_public_key(&self) -> Result<VapidPublicKey, Self::Error> {
        Ok(self
            .get_vapid_keypair()
            .await?
            .public()
            .map_err(Error::VapidError)?)
    }
}

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
pub trait SubscriptionReadStore: WithError + Send + Sync + 'static {
    async fn get_subscriptions(&self, topic: &str) -> Result<Vec<Subscription>, Self::Error>;
    async fn get_subscription(&self, id: &str) -> Result<Subscription, Self::Error>;
}

#[async_trait]
pub trait SubscriptionWriteStore: WithError + Send + Sync + 'static {
    /// Returns whether a subscription under the id already existed
    async fn upsert_subscription(&self, sub: Subscription) -> Result<bool, Self::Error>;
    async fn delete_subscription(&self, id: &str) -> Result<(), Self::Error>;
}

pub trait SubscriptionStore:
    SubscriptionReadStore + SubscriptionWriteStore + VapidKeyStore
{
}
