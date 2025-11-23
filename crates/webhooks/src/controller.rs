use tokio::sync::mpsc::Receiver;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{engine::general_purpose, Engine as _};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use rustical_types::WebhookEvent;

// Assume WebhookStore is defined elsewhere and provides endpoint lookup
use rustical_store::WebhookSubscriptionStore;

/// Controller that receives WebhookEvents and processes them
pub struct WebhookController<S: WebhookSubscriptionStore> {
	receiver: Receiver<WebhookEvent>,
	store: Arc<S>,
}

struct RetryItem {
	endpoint: String,
	payload: String,
	attempts: u32,
	max_attempts: u32,
	backoff: Duration,
}

type HmacSha256 = Hmac<Sha256>;

fn sign_payload(payload: &str, secret: &str) -> String {
	let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
	mac.update(payload.as_bytes());
	let result = mac.finalize().into_bytes();
	general_purpose::STANDARD.encode(result)
}

impl<S: WebhookSubscriptionStore> WebhookController<S> {
	pub fn new(receiver: Receiver<WebhookEvent>, store: Arc<S>) -> Self {
		Self { receiver, store }
	}

	/// Start processing events (to be expanded with dispatch, retry, etc.)
	pub async fn run(mut self) {

		while let Some(event) = self.receiver.recv().await {
			let subscriptions = match self
                .store
                .get_subscriptions(&event.resource_type(), &event.resource_id())
                .await
            {
                Ok(list) => list,
                Err(err) => {
                    eprintln!("Failed to load subscriptions: {:?}", err);
                    vec![]
                }
            };
			println!("Received event: {:?}", event);
			println!("Found subscribers: {:?}", subscriptions);
			let payload = event.to_payload_json();
            
			for subscription in subscriptions {
				let mut signature = None;
				if let Some(secret) = &subscription.secret_key {
					signature = Some(sign_payload(&payload.to_string(), secret));
					println!("Signing payload for {} with secret", subscription.target_url);
				}
				let endpoint = subscription.target_url.clone();
				let payload_str = payload.to_string();
				let max_attempts = 5;
				let initial_backoff = Duration::from_secs(2);
				let retry_item = RetryItem {
					endpoint: endpoint.clone(),
					payload: payload_str.clone(),
					attempts: 0,
					max_attempts,
					backoff: initial_backoff,
				};
				tokio::spawn(Self::send_with_retry(retry_item, signature));
			}
		}
        
	}

	async fn send_with_retry(mut item: RetryItem, signature: Option<String>) {
		let client = reqwest::Client::new();
		loop {
			let mut req = client.post(&item.endpoint)
				.header("Content-Type", "application/json");
			if let Some(sig) = &signature {
				req = req.header("X-Signature", sig);
			}
			let res = req.body(item.payload.clone()).send().await;
			match res {
				Ok(resp) if resp.status().is_success() => {
					println!("Webhook delivered to {}", item.endpoint);
					break;
				}
				Ok(resp) => {
					println!("Webhook delivery failed to {}: {}", item.endpoint, resp.status());
				}
				Err(e) => {
					println!("Webhook delivery error to {}: {}", item.endpoint, e);
				}
			}
			item.attempts += 1;
			if item.attempts >= item.max_attempts {
				println!("Max retry attempts reached for {}", item.endpoint);
				break;
			}
			println!("Retrying {} in {:?} (attempt {})", item.endpoint, item.backoff, item.attempts);
			sleep(item.backoff).await;
			item.backoff *= 2;
		}
	}
}
