pub mod middleware;
mod principal;
use crate::error::Error;
use async_trait::async_trait;

mod principal_type;
pub use principal_type::*;

pub use principal::{AppToken, Principal};

#[async_trait]
pub trait AuthenticationProvider: Send + Sync + 'static {
    async fn get_principals(&self) -> Result<Vec<Principal>, crate::Error>;
    async fn get_principal(&self, id: &str) -> Result<Option<Principal>, crate::Error>;
    async fn remove_principal(&self, id: &str) -> Result<(), crate::Error>;
    async fn insert_principal(&self, user: Principal, overwrite: bool) -> Result<(), crate::Error>;
    async fn validate_password(
        &self,
        user_id: &str,
        password: &str,
    ) -> Result<Option<Principal>, Error>;
    async fn validate_app_token(
        &self,
        user_id: &str,
        token: &str,
    ) -> Result<Option<Principal>, Error>;
    /// Returns a token identifier
    async fn add_app_token(
        &self,
        user_id: &str,
        name: String,
        token: String,
    ) -> Result<String, Error>;
    async fn remove_app_token(&self, user_id: &str, token_id: &str) -> Result<(), Error>;
    async fn get_app_tokens(&self, principal: &str) -> Result<Vec<AppToken>, Error>;

    async fn add_membership(&self, principal: &str, member_of: &str) -> Result<(), Error>;
    async fn remove_membership(&self, principal: &str, member_of: &str) -> Result<(), Error>;
    async fn list_members(&self, principal: &str) -> Result<Vec<String>, Error>;
}

pub use middleware::AuthenticationMiddleware;
