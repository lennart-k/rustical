use serde::{Deserialize, Serialize};

const fn default_true() -> bool {
    true
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct FrontendConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub allow_password_login: bool,
    /// Send visitors of the login page straight to the OIDC provider instead of
    /// showing the "Login with <provider>" button. Only takes effect when OIDC is
    /// configured and `allow_password_login` is `false`, i.e. when the button is
    /// the only thing the page could offer.
    #[serde(default)]
    pub oidc_auto_redirect: bool,
}

impl Default for FrontendConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_password_login: true,
            oidc_auto_redirect: false,
        }
    }
}
