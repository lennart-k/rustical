use crate::{Error, Secret};
use argon2::{Argon2, password_hash::PasswordHasher};
use password_hash::phc::SaltString;

/// Hashes a login password with Argon2id and the default parameters.
///
/// # Errors
///
/// Returns [`Error::PasswordHash`] if password hashing fails.
pub fn hash_password(password: &str) -> Result<Secret<String>, Error> {
    let salt = SaltString::generate();

    Argon2::default()
        .hash_password_with_salt(password.as_bytes(), salt.as_bytes())
        .map(|hash| Secret::from(hash.to_string()))
        .map_err(|_| Error::PasswordHash)
}
