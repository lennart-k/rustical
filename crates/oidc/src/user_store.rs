use async_trait::async_trait;
use axum::response::IntoResponse;

#[async_trait]
pub trait UserStore: 'static + Send + Sync + Clone {
    type Error: IntoResponse;

    async fn user_exists(&self, id: &str) -> Result<bool, Self::Error>;
    /// Ensures a user with id and memberhips exists
    /// Note that memberships is ONLY ADDITIVE
    async fn ensure_user(&self, id: &str, memberships: &[&str]) -> Result<(), Self::Error>;
}
