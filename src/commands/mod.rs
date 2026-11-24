use std::io::IsTerminal;

use crate::config::{
    Config, DataStoreConfig, DavPushConfig, HttpConfig, MaintenanceConfig, NextcloudLoginConfig,
    SqliteDataStoreConfig, TracingConfig,
};
use anyhow::anyhow;
use clap::Parser;
use rustical_caldav::CalDavConfig;
use rustical_frontend::{AdminConfig, FrontendConfig};

pub mod app_token;
mod hash_password;
mod health;
pub mod membership;
pub mod principals;

pub use hash_password::{HashPasswordArgs, cmd_hash_password};
pub use health::{HealthArgs, cmd_health};
pub use principals::{PrincipalsArgs, cmd_principals};

/// Asks for a password via terminal prompt, or reads it from stdin when piped.
fn prompt_password_or_read_stdio() -> anyhow::Result<String> {
    let stdin = std::io::stdin();
    let password = if stdin.is_terminal() {
        rpassword::prompt_password_with_config(
            "Type in a password: ",
            rpassword::ConfigBuilder::new()
                .password_feedback_mask('*')
                .build(),
        )?
    } else {
        let mut password = String::new();
        let _ = stdin.read_line(&mut password)?;
        password.trim_end().to_string()
    };

    if password.is_empty() {
        return Err(anyhow!("Password is empty"));
    }

    Ok(password)
}

#[derive(Debug, Parser)]
pub struct GenConfigArgs {}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn cmd_gen_config(_args: GenConfigArgs) -> anyhow::Result<()> {
    let config = Config {
        http: HttpConfig::default(),
        caldav: CalDavConfig::default(),
        data_store: DataStoreConfig::Sqlite(SqliteDataStoreConfig {
            db_url: "/var/lib/rustical/db.sqlite3".to_owned(),
            run_repairs: true,
            skip_broken: true,
        }),
        tracing: TracingConfig::default(),
        frontend: FrontendConfig {
            enabled: true,
            allow_password_login: true,
            admin: AdminConfig::default(),
        },
        oidc: None,
        dav_push: DavPushConfig::default(),
        nextcloud_login: NextcloudLoginConfig::default(),
        maintenance: MaintenanceConfig::default(),
    };
    let generated_config = toml::to_string(&config)?;
    println!("{generated_config}");
    Ok(())
}
