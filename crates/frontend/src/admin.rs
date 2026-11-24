use crate::config::{AdminConfig, BootstrapAdmins, InitialAdminConfig};
use rustical_store::{
    Error, Secret,
    auth::{AuthenticationProvider, Principal, PrincipalType, hash_password},
};
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, Item};
use tracing::{info, warn};

/// Creates the configured initial admin principals and adds them to the admin group.
///
/// Existing principals are never modified, so admins can safely revoke their own access;
/// their admin group membership is re-established on every bootstrap run.
///
/// When the `bootstrap_admins` option ran as `once` or a plaintext `password` was used,
/// the config file is updated accordingly (`bootstrap_admins = "no"`, `password_hash`
/// instead of `password`). Config file updates are best-effort: if they fail, a warning
/// tells the operator to update the file manually and startup continues.
///
/// # Errors
///
/// Returns an error if reading or writing principals or memberships fails,
/// or if password hashing fails.
pub async fn bootstrap_admin<AP: AuthenticationProvider + ?Sized>(
    auth_provider: &AP,
    config: &AdminConfig,
    config_path: &Path,
) -> Result<(), Error> {
    if !config.enabled() {
        return Ok(());
    }

    let mut missing_admins = Vec::new();
    for admin in &config.initial_admins {
        if auth_provider.get_principal(&admin.id).await?.is_none() {
            missing_admins.push(admin);
        }
    }

    if !should_bootstrap(config, &missing_admins) {
        return Ok(());
    }

    // Create all missing initial admins first ...
    let mut password_hashes = Vec::new();
    for admin in &missing_admins {
        let password = match (&admin.password, &admin.password_hash) {
            (Some(password), _) => {
                let hash = hash_password(password)?;
                password_hashes.push((admin.id.clone(), hash.as_ref().clone()));
                Some(hash)
            }
            (None, Some(password_hash)) => Some(Secret::from(password_hash.clone())),
            (None, None) => None,
        };

        let principal = Principal {
            id: admin.id.clone(),
            displayname: admin.displayname.clone(),
            principal_type: PrincipalType::Individual,
            password,
            memberships: Vec::new(),
        };

        match auth_provider.insert_principal(principal, false).await {
            Ok(()) => info!("Created initial admin principal {}", admin.id),
            Err(Error::AlreadyExists) => {}
            Err(error) => return Err(error),
        }
    }

    // ... then make sure the admin group exists and every initial admin is a member of it
    if let Some(admin_group) = &config.admin_group {
        ensure_group(auth_provider, admin_group).await?;
        for admin in &config.initial_admins {
            auth_provider.add_membership(&admin.id, admin_group).await?;
        }
    }

    update_config_file(
        config_path,
        matches!(config.bootstrap_admins, Some(BootstrapAdmins::Once)),
        &password_hashes,
    );

    if !auth_provider
        .get_principals()
        .await?
        .iter()
        .any(|principal| config.is_admin(principal))
    {
        warn!(
            "Admin settings are enabled but no principal is a member of the configured admin group. \
             Use the initial admin bootstrap or membership assignment (for example via OIDC) to grant access."
        );
    }

    Ok(())
}

/// Whether the bootstrap should run for the given configuration state.
///
/// Without an explicit `bootstrap_admins` setting it runs whenever a configured
/// initial admin does not exist yet, and does nothing while all of them exist.
#[must_use]
const fn should_bootstrap(config: &AdminConfig, missing_admins: &[&InitialAdminConfig]) -> bool {
    match config.bootstrap_admins {
        Some(BootstrapAdmins::No) => false,
        Some(BootstrapAdmins::Once | BootstrapAdmins::Always) => true,
        None => !missing_admins.is_empty(),
    }
}

/// Best-effort config file maintenance after a bootstrap run.
fn update_config_file(
    config_path: &Path,
    set_bootstrap_no: bool,
    password_hashes: &[(String, String)],
) {
    if !set_bootstrap_no && password_hashes.is_empty() {
        return;
    }

    match try_update_config_file(config_path, set_bootstrap_no, password_hashes) {
        Ok(true) => info!("Updated config file {}", config_path.display()),
        Ok(false) => {}
        Err(error) => warn!(
            "Could not update the config file {}: {error}. Please update it yourself: set \
             `bootstrap_admins = \"no\"` after a one-time bootstrap and replace initial admin \
             `password` entries with the printed `password_hash`.",
            config_path.display()
        ),
    }
}

fn try_update_config_file(
    config_path: &Path,
    set_bootstrap_no: bool,
    password_hashes: &[(String, String)],
) -> Result<bool, String> {
    let raw = fs::read_to_string(config_path).map_err(|error| error.to_string())?;
    let mut doc: DocumentMut = raw
        .parse::<DocumentMut>()
        .map_err(|error| error.to_string())?;

    if doc.get("frontend").is_none() {
        doc["frontend"] = toml_edit::table();
    }
    let frontend = doc
        .get_mut("frontend")
        .and_then(Item::as_table_mut)
        .ok_or("the `frontend` setting is not a table")?;

    if frontend.get("admin").is_none() {
        frontend["admin"] = toml_edit::table();
    }
    let admin = frontend
        .get_mut("admin")
        .and_then(Item::as_table_mut)
        .ok_or("`frontend.admin` is not a table")?;

    let mut changed = set_bootstrap_no;

    if set_bootstrap_no {
        admin["bootstrap_admins"] = toml_edit::value("no");
    }

    if !password_hashes.is_empty() {
        let Some(Item::ArrayOfTables(admins)) = admin.get_mut("initial_admins") else {
            return Err("`frontend.admin.initial_admins` is not an array of tables".to_owned());
        };

        for entry in admins.iter_mut() {
            let Some(id) = entry.get("id").and_then(Item::as_str) else {
                continue;
            };
            let Some((_, hash)) = password_hashes.iter().find(|(admin_id, _)| admin_id == id)
            else {
                continue;
            };

            if entry.contains_key("password") {
                entry.remove("password");
                entry["password_hash"] = toml_edit::value(hash.clone());
                changed = true;
            }
        }
    }

    if changed {
        fs::write(config_path, doc.to_string()).map_err(|error| error.to_string())?;
    }

    Ok(changed)
}

async fn ensure_group<AP: AuthenticationProvider + ?Sized>(
    auth_provider: &AP,
    id: &str,
) -> Result<(), Error> {
    if auth_provider.get_principal(id).await?.is_some() {
        return Ok(());
    }

    match auth_provider
        .insert_principal(
            Principal {
                id: id.to_owned(),
                displayname: None,
                principal_type: PrincipalType::Group,
                password: None,
                memberships: Vec::new(),
            },
            false,
        )
        .await
    {
        Ok(()) => {
            info!("Created admin group {}", id);
            Ok(())
        }
        Err(Error::AlreadyExists) => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("rustical-admin-test-{}", uuid::Uuid::new_v4()));
        path
    }

    #[test]
    fn config_update_sets_bootstrap_no_and_keeps_comments() {
        let path = temp_path();
        fs::write(
            &path,
            "[frontend.admin]\nadmin_group = \"admin_group\" # keep me\nbootstrap_admins = \"once\"\n",
        )
        .unwrap();

        assert!(try_update_config_file(&path, true, &[]).unwrap());

        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("bootstrap_admins = \"no\""));
        assert!(after.contains("# keep me"));
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn config_update_replaces_password_with_hash() {
        let path = temp_path();
        fs::write(
            &path,
            "[frontend.admin]\nadmin_group = \"admin_group\"\n\n[[frontend.admin.initial_admins]]\nid = \"admin\"\npassword = \"secret\"\n\n[[frontend.admin.initial_admins]]\nid = \"other\"\npassword = \"untouched\"\n",
        )
        .unwrap();

        assert!(
            try_update_config_file(
                &path,
                false,
                &[("admin".to_owned(), "argon2$hash".to_owned())]
            )
            .unwrap()
        );

        let after = fs::read_to_string(&path).unwrap();
        assert!(after.contains("password_hash = \"argon2$hash\""));
        assert!(!after.contains("password = \"secret\""));
        assert!(after.contains("password = \"untouched\""));
        fs::remove_file(&path).unwrap();
    }

    #[test]
    fn config_update_fails_gracefully_without_file() {
        assert!(try_update_config_file(Path::new("/nonexistent/config.toml"), true, &[]).is_err());
    }
}
