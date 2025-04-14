pub mod middleware;
pub mod toml_user_store;
pub mod user;
use crate::error::Error;
use async_trait::async_trait;

#[async_trait]
pub trait AuthenticationProvider: 'static {
    async fn get_principal(&self, id: &str) -> Result<Option<User>, crate::Error>;
    async fn insert_principal(&self, user: User) -> Result<(), crate::Error>;
    async fn validate_user_token(&self, user_id: &str, token: &str) -> Result<Option<User>, Error>;
    /// Returns a token identifier
    async fn add_app_token(
        &self,
        user_id: &str,
        name: String,
        token: String,
    ) -> Result<String, Error>;
    async fn remove_app_token(&self, user_id: &str, token_id: &str) -> Result<(), Error>;
}

pub use middleware::AuthenticationMiddleware;
pub use toml_user_store::{TomlPrincipalStore, TomlUserStoreConfig};
pub use user::User;
