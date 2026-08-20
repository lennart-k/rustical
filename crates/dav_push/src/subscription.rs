use chrono::{DateTime, NaiveDateTime, TimeZone};
use web_push::SubscriptionKeys;

pub struct Subscription {
    pub id: String,
    pub topic: String,
    // Naive because sqlite has no concept of timezones
    // In reality, this is UTC
    pub expiration: NaiveDateTime,
    pub push_resource: String,
    pub public_key: String,
    // Currently, only p256dh
    pub public_key_type: String,
    pub auth_secret: String,
}

impl Subscription {
    #[must_use]
    pub fn is_expired(&self, now: &DateTime<impl TimeZone>) -> bool {
        self.expiration < now.naive_utc()
    }
}

impl From<Subscription> for web_push::SubscriptionInfo {
    fn from(value: Subscription) -> Self {
        Self {
            endpoint: value.push_resource,
            keys: SubscriptionKeys {
                p256dh: value.public_key,
                auth: value.auth_secret,
            },
        }
    }
}
