use rustical_store::auth::Principal;
use serde::{Deserialize, Serialize};

const fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct InitialAdminConfig {
    pub id: String,
    #[serde(default)]
    pub displayname: Option<String>,
    /// Existing Argon2 password hash.
    #[serde(default)]
    pub password_hash: Option<String>,
    /// Plaintext password used to bootstrap a new admin.
    /// This is only used when the configured principal does not exist yet.
    #[serde(default)]
    pub password: Option<String>,
}

/// Controls when the initial admins are bootstrapped.
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BootstrapAdmins {
    /// Never run the initial admin bootstrap.
    No,
    /// Bootstrap now and set this option back to `no` in the config file.
    Once,
    /// Bootstrap on every startup, re-adding initial admins to the admin group.
    Always,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields, default)]
pub struct AdminConfig {
    #[serde(default)]
    pub admin_group: Option<String>,
    #[serde(default)]
    pub initial_admins: Vec<InitialAdminConfig>,
    /// If unset, bootstraps whenever a configured initial admin does not exist yet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap_admins: Option<BootstrapAdmins>,
}

impl AdminConfig {
    #[must_use]
    pub fn is_admin(&self, principal: &Principal) -> bool {
        self.is_member_of_admin_group(principal)
    }

    /// The admin frontend is enabled as soon as an admin group is configured.
    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.admin_group.is_some()
    }

    #[must_use]
    pub fn is_member_of_admin_group(&self, principal: &Principal) -> bool {
        self.admin_group
            .as_deref()
            .is_some_and(|admin_group| principal.is_member_of(admin_group))
    }
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FrontendConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub allow_password_login: bool,
    #[serde(default)]
    pub admin: AdminConfig,
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_password_login: true,
            admin: AdminConfig::default(),
        }
    }
}
