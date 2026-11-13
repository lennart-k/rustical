pub mod middleware;
mod principal;
use crate::error::Error;
use async_trait::async_trait;

mod principal_type;
pub use principal_type::*;

mod error;
pub use error::UnauthorizedError;

pub use principal::{AppToken, Principal};

/// The `AuthenticationProvider` is the principal store for rustical.
#[async_trait]
pub trait AuthenticationProvider: Send + Sync + 'static {
    /// Returns a list of all principals
    async fn get_principals(&self) -> Result<Vec<Principal>, Error>;

    /// Returns a principal by its `id` as an `Option<Principal>`.
    /// If the principal does not exist `Ok(None)` is returned.
    async fn get_principal(&self, id: &str) -> Result<Option<Principal>, Error>;

    async fn remove_principal(&self, id: &str) -> Result<(), Error>;

    /// Inserts a principal and upserts it if `overwrite=true`.
    /// Ignores `Principal.membership` field which is instead managed
    /// via the `*_membership` methods.
    async fn insert_principal(&self, user: Principal, overwrite: bool) -> Result<(), Error>;

    /// Validates a password input for a principal.
    /// If the password input is correct returns `Ok(Some(principal))`.
    async fn validate_password(
        &self,
        user_id: &str,
        password_input: &str,
    ) -> Result<Option<Principal>, Error> {
        let user: Principal = match self.get_principal(user_id).await? {
            Some(user) => user,
            None => return Ok(None),
        };
        let Some(password) = &user.password else {
            return Ok(None);
        };

        if password_auth::verify_password(password_input, password.as_ref()).is_ok() {
            return Ok(Some(user));
        }
        Ok(None)
    }

    /// Validates an app token for a given principal id.
    async fn validate_app_token(
        &self,
        user_id: &str,
        token: &str,
    ) -> Result<Option<Principal>, Error> {
        // Allow to specify the token id to use to make validation faster
        // Doesn't match the whole length of the token id to keep the length in bounds
        // Example: asd_selgkh
        // where the app token id starts with asd and its value is selgkh
        let (token_id_prefix, token) = token.split_once('_').unwrap_or(("", token));

        for app_token in &self.get_app_tokens(user_id).await? {
            // Wrong token id
            if !app_token.id.starts_with(token_id_prefix) {
                continue;
            }
            if password_auth::verify_password(token, app_token.token.as_ref()).is_ok() {
                // Only bookkeeping: a failure to record the use must not reject a valid token.
                if let Err(err) = self.touch_app_token(user_id, &app_token.id).await {
                    tracing::warn!("Could not record the use of an app token: {err}");
                }
                return self.get_principal(user_id).await;
            }
        }
        Ok(None)
    }

    /// Returns a token identifier
    async fn add_app_token(
        &self,
        user_id: &str,
        name: String,
        token: String,
    ) -> Result<String, Error>;
    async fn remove_app_token(&self, user_id: &str, token_id: &str) -> Result<(), Error>;

    /// Records that an app token has just authenticated a request.
    /// Called by `validate_app_token`; the default implementation does nothing.
    async fn touch_app_token(&self, _user_id: &str, _token_id: &str) -> Result<(), Error> {
        Ok(())
    }

    async fn get_app_tokens(&self, principal: &str) -> Result<Vec<AppToken>, Error>;

    async fn add_membership(&self, principal: &str, member_of: &str) -> Result<(), Error>;

    async fn remove_membership(&self, principal: &str, member_of: &str) -> Result<(), Error>;

    async fn list_members(&self, principal: &str) -> Result<Vec<String>, Error>;
}

pub use middleware::AuthenticationMiddleware;

#[cfg(test)]
mod tests {
    #[rstest::rstest]
    // 0.18.0 example
    #[case(
        "ZZDlOOeqqWj6DKz2EoS8fvT54YITxKXYnpkZOc2jbTCOwncR0Zkpe62FV10Vdq3F",
        "$pbkdf2-sha512$i=1000,l=32$ZLThcnZeCH7TodsV4A60aA$KogO//I83AZPnMLXOk9eLGFr3KTR4doZq9bDOSojCec"
    )]
    #[case(
        "yZXlwdGKJrvpOYcgYNmvBt9vEoGlvqHhzyD800cZ1OxuIpdHxvBBqA4quqeADpjA",
        "$pbkdf2-sha512$i=1000,l=32$Akwa7kQ7UWX17cnliam8sw$b5p2wGfe933MfT0FpBfiFm/lFXK9CGnNxpBNU69UtQU"
    )]
    // 0.16.0
    #[case(
        "vbZ3DbFCzV0myGRqb6RWD7lm5P4rfArk3EErY9YYwYp4VDr3Y0Rw9eoTieBa03Q5",
        "$pbkdf2-sha512$i=1000,l=32$hkuHqLKRTSXppYS+dTu9ZA$ee9g4khayH7RK44Qz33uMidx+yItLVuuOok5qSeHCT4"
    )]
    // 0.15.0
    #[case(
        "YSLcThYIhtFuoebHmK8n3UiqLVi86jGwGGblGfODTNyqE3C92TK16vwHufmxk0Rw",
        "$pbkdf2-sha512$i=1000,l=32$3WaZ7rhZLxF+ddO1geEpdA$buXL9nxbXqUEEzgrt/bGNMSMcbP03yHjKbkXYVFtEYY"
    )]
    // 0.14.0
    #[case(
        "HTTuayJ0zw4nOKA7LFZ0EsE2OtyjedIIsWbpt9FhJhpjBknd6nxDOpBsCZULz77d",
        "$pbkdf2-sha512$i=1000,l=32$CBYnva+LjWqGhfvMz2SyLQ$WthPDeNIKKA6B+CdD5M6MwhOTxBlGmEOTbgWblhG6uM"
    )]
    // 0.13.0
    #[case(
        "ZPbWO4eexd8ki5GDWDFLtD6Vx1aOfXbLVU1BnbteWBCrNUffvJ58cQMxyY3BHyMR",
        "$pbkdf2-sha512$i=1000,l=32$5Gz4A2JIZpbM2R79K8gxoA$Na92vSJFFCZCZtE3E/lKP10gGweaUNO7eknVKDrD2/U"
    )]
    // 0.12.0
    #[case(
        "S7h7hU859y9zyCZ5MeeBSQxyGy1ckAE8NMiNBCYBcpBLqP2KL3TcPNDgArg6jMNf",
        "$pbkdf2-sha256$i=2,l=32$kq9i4C+MQhxKqHpFlmQxhQ$dOBHgWQ28Z2yEgpDwFOJZGYFI/tfCFwMqu6fKAJ+sVE"
    )]
    fn test_password_auth_validates_tokens(#[case] token: &str, #[case] hash: &str) {
        password_auth::verify_password(token, hash).unwrap();
    }
}
