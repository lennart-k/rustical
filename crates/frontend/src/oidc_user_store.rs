use async_trait::async_trait;
use rustical_oidc::UserStore;
use rustical_store::auth::{AuthenticationProvider, Principal, PrincipalType};
use std::sync::Arc;

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

    /// Ensures a principal with id exists.
    /// Also adds memberships, but does NOT remove previous ones
    /// If assigning a membership fails (e.g. due to the principal not existing),
    /// the method will not fail but only log an error.
    async fn ensure_user(&self, id: &str, memberships: &[&str]) -> Result<(), Self::Error> {
        // Ensure user exists at all
        match self
            .0
            .insert_principal(
                Principal {
                    id: id.to_owned(),
                    displayname: None,
                    principal_type: PrincipalType::default(),
                    password: None,
                    memberships: vec![],
                },
                false,
            )
            .await
        {
            Ok(()) | Err(rustical_store::Error::AlreadyExists) => {}
            Err(err) => return Err(err),
        }

        // Add additional memberships
        let Some(user) = self.0.get_principal(id).await? else {
            return Err(rustical_store::Error::NotFound);
        };
        for membership in memberships {
            if !user.memberships().contains(membership)
                && let Err(err) = self.0.add_membership(id, membership).await
            {
                tracing::error!(
                    "Failed to assign membership {membership} to principal {id}: {err}"
                );
            }
        }

        Ok(())
    }
}
