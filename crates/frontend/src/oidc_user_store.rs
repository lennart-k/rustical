use std::sync::Arc;

use async_trait::async_trait;
use rustical_oidc::UserStore;
use rustical_store::auth::{AuthenticationProvider, User};

pub struct OidcUserStore<AP: AuthenticationProvider>(pub Arc<AP>);

impl<AP: AuthenticationProvider> Clone for OidcUserStore<AP> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait]
impl<AP: AuthenticationProvider> UserStore for OidcUserStore<AP> {
    type Error = rustical_store::Error;

    async fn user_exists(&self, id: &str) -> Result<bool, Self::Error> {
        Ok(self.0.get_principal(id).await?.is_some())
    }

    async fn insert_user(&self, id: &str) -> Result<(), Self::Error> {
        self.0
            .insert_principal(
                User {
                    id: id.to_owned(),
                    displayname: None,
                    principal_type: Default::default(),
                    password: None,
                    memberships: vec![],
                },
                false,
            )
            .await
    }
}
