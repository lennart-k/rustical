use async_trait::async_trait;
use axum::response::IntoResponse;

#[async_trait]
pub trait UserStore: 'static + Send + Sync + Clone {
    type Error: IntoResponse;

    async fn user_exists(&self, id: &str) -> Result<bool, Self::Error>;
    /// Ensures a user with id and memberhips exists
    /// Note that memberships is ONLY ADDITIVE
    /// `email` is the e-mail address the identity provider asserts: it becomes the user's (also
    /// if another user had it); without one the user's address stays as it is.
    async fn ensure_user(
        &self,
        id: &str,
        memberships: &[&str],
        email: Option<&str>,
    ) -> Result<(), Self::Error>;
}
