use super::prompt_password_or_read_stdio;
use anyhow::{Result, anyhow};
use clap::Parser;
use rustical_store::auth::hash_password;

#[derive(Parser, Debug)]
pub struct HashPasswordArgs {
    /// Password to hash. Omit the argument (or use "-") to be asked for it.
    pub password: Option<String>,
}

/// Prints an Argon2 hash for use as `password_hash` of a `[[frontend.admin.initial_admins]]` entry.
#[allow(clippy::missing_errors_doc)]
pub fn cmd_hash_password(args: HashPasswordArgs) -> Result<()> {
    let password = match args.password {
        Some(password) if password != "-" => password,
        _ => prompt_password_or_read_stdio()?,
    };

    if password.is_empty() {
        return Err(anyhow!("Password is empty"));
    }

    let hash = hash_password(&password).map_err(anyhow::Error::from)?;
    println!("{}", hash.as_ref());

    Ok(())
}
