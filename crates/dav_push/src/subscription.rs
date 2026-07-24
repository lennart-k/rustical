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
