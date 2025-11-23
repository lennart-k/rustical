use serde_json::{json, Value};

#[derive(Debug)]
pub enum WebhookEvent {
    CalendarObjectUpsert {
        resource_id: String,
        object_uid: String,
        timestamp: i64, 
        ics_data: String, // full ICS data
    },
    CalendarObjectDelete {
        resource_id: String,
        object_uid: String,
        timestamp: i64
    },
    CalendarObjectRestore {
        resource_id: String,
        object_uid: String,
        timestamp: i64
    },
    AddressbookObjectUpsert {
        resource_id: String,
        object_uid: String,
        timestamp: i64, 
        vcf_data: String, // full VCF data
    },
    AddressbookObjectDelete {
        resource_id: String,
        object_uid: String,
        timestamp: i64
    },
    AddressbookObjectRestore {
        resource_id: String,
        object_uid: String,
        timestamp: i64
    },
}

#[derive(Debug)]
pub struct WebhookSubscription {
    pub id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub target_url: String,
    pub secret_key: Option<String>,
}


impl WebhookEvent {
    pub fn event_type(&self) -> &str {
        match self {
            WebhookEvent::CalendarObjectUpsert { .. } => "Upsert",
            WebhookEvent::CalendarObjectDelete { .. } => "Delete",
            WebhookEvent::CalendarObjectRestore { .. } => "Restore",
            WebhookEvent::AddressbookObjectUpsert { .. } => "Upsert",
            WebhookEvent::AddressbookObjectDelete { .. } => "Delete",
            WebhookEvent::AddressbookObjectRestore { .. } => "Restore",
        }
    }

    /// Build the webhook JSON payload with all required fields
    pub fn to_payload_json(&self) -> Value {
        match self {
            WebhookEvent::CalendarObjectUpsert { resource_id, object_uid, timestamp, ics_data } => {
                json!({
                    "event_type": self.event_type(),
                    "resource_type": self.resource_type(),
                    "resource_id": resource_id,
                    "object_uid": object_uid,
                    "timestamp": timestamp,
                    "ics_data": ics_data
                })
            }
            WebhookEvent::CalendarObjectDelete { resource_id, object_uid, timestamp } => {
                json!({
                    "event_type": self.event_type(),
                    "resource_type": self.resource_type(),
                    "resource_id": resource_id,
                    "object_uid": object_uid,
                    "timestamp": timestamp
                })
            }
            WebhookEvent::CalendarObjectRestore { resource_id, object_uid, timestamp } => {
                json!({
                    "event_type": self.event_type(),
                    "resource_type": self.resource_type(),
                    "resource_id": resource_id,
                    "object_uid": object_uid,
                    "timestamp": timestamp
                })
            }
            WebhookEvent::AddressbookObjectUpsert { resource_id, object_uid, timestamp, vcf_data } => {
                json!({
                    "event_type": self.event_type(),
                    "resource_type": self.resource_type(),
                    "resource_id": resource_id,
                    "object_uid": object_uid,
                    "timestamp": timestamp,
                    "vcf_data": vcf_data
                })
            }
            WebhookEvent::AddressbookObjectDelete { resource_id, object_uid, timestamp } => {
                json!({
                    "event_type": self.event_type(),
                    "resource_type": self.resource_type(),
                    "resource_id": resource_id,
                    "object_uid": object_uid,
                    "timestamp": timestamp
                })
            }
            WebhookEvent::AddressbookObjectRestore { resource_id, object_uid, timestamp } => {
                json!({
                    "event_type": self.event_type(),
                    "resource_type": self.resource_type(),
                    "resource_id": resource_id,
                    "object_uid": object_uid,
                    "timestamp": timestamp
                })
            }
        }
    }

    pub fn resource_type(&self) -> &'static str {
        match self {
            WebhookEvent::CalendarObjectUpsert { .. } => "calendar",
            WebhookEvent::CalendarObjectDelete { .. } => "calendar",
            WebhookEvent::CalendarObjectRestore { .. } => "calendar",
            WebhookEvent::AddressbookObjectUpsert { .. } => "addressbook",
            WebhookEvent::AddressbookObjectDelete { .. } => "addressbook",
            WebhookEvent::AddressbookObjectRestore { .. } => "addressbook",
        }
    }

    pub fn resource_id(&self) -> &str {
        match self {
            WebhookEvent::CalendarObjectUpsert { resource_id, .. } =>
                resource_id,
            WebhookEvent::CalendarObjectDelete { resource_id, .. } =>
                resource_id,
            WebhookEvent::CalendarObjectRestore { resource_id, .. } =>
                resource_id,
            WebhookEvent::AddressbookObjectUpsert { resource_id, .. } =>
                resource_id,
            WebhookEvent::AddressbookObjectDelete { resource_id, .. } =>
                resource_id,
            WebhookEvent::AddressbookObjectRestore { resource_id, .. } =>
                resource_id,
        }
    }

    pub fn object_uid(&self) -> &str {
        match self {
            WebhookEvent::CalendarObjectUpsert { object_uid, .. } =>
                object_uid,
            WebhookEvent::CalendarObjectDelete { object_uid, .. } =>
                object_uid,
            WebhookEvent::CalendarObjectRestore { object_uid, .. } =>
                object_uid,
            WebhookEvent::AddressbookObjectUpsert { object_uid, .. } =>
                object_uid,
            WebhookEvent::AddressbookObjectDelete { object_uid, .. } =>
                object_uid,
            WebhookEvent::AddressbookObjectRestore { object_uid, .. } =>
                object_uid,
        }
    }

    pub fn timestamp(&self) -> i64 {
        match self {
            WebhookEvent::CalendarObjectUpsert { timestamp, .. } =>
                *timestamp,
            WebhookEvent::CalendarObjectDelete { timestamp, .. } =>
                *timestamp,
            WebhookEvent::CalendarObjectRestore { timestamp, .. } =>
                *timestamp,
            WebhookEvent::AddressbookObjectUpsert { timestamp, .. } =>
                *timestamp,
            WebhookEvent::AddressbookObjectDelete { timestamp, .. } =>
                *timestamp,
            WebhookEvent::AddressbookObjectRestore { timestamp, .. } =>
                *timestamp,
        }
    }
}