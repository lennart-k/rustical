pub mod middleware;
pub mod toml_user_store;
pub mod user;
use crate::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait AuthenticationProvider: 'static {
    async fn get_principal(&self, id: &str) -> Result<Option<User>, crate::Error>;
    async fn validate_user_token(&self, user_id: &str, token: &str) -> Result<Option<User>, Error>;
    async fn add_app_token(&self, user_id: &str, name: String, token: String) -> Result<(), Error>;
}

pub use middleware::AuthenticationMiddleware;
pub use toml_user_store::{TomlPrincipalStore, TomlUserStoreConfig};
pub use user::User;
