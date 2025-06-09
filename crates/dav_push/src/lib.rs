mod extension;
pub mod notifier;
mod prop;
pub mod register;
use derive_more::Constructor;
pub use extension::*;
pub use prop::*;
use rustical_store::{CollectionOperation, SubscriptionStore};
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tracing::error;

#[derive(Debug, Constructor)]
pub struct DavPushController<S: SubscriptionStore> {
    allowed_push_servers: Option<Vec<String>>,
    sub_store: Arc<S>,
}

impl<S: SubscriptionStore> DavPushController<S> {
    pub async fn notifier(&self, mut recv: Receiver<CollectionOperation>) {
        while let Some(message) = recv.recv().await {
            let subscribers = match self.sub_store.get_subscriptions(&message.topic).await {
                Ok(subs) => subs,
                Err(err) => {
                    error!("{err}");
                    continue;
                }
            };
        }
    }
}
