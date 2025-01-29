pub mod middleware;
pub mod static_user_store;
pub mod user;
use crate::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait AuthenticationProvider: 'static {
    async fn validate_user_token(&self, user_id: &str, token: &str) -> Result<Option<User>, Error>;
}

pub use middleware::AuthenticationMiddleware;
pub use static_user_store::{StaticUserStore, StaticUserStoreConfig};
pub use user::User;
