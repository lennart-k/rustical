use crate::config::{
    Config, DataStoreConfig, DavPushConfig, HttpConfig, NextcloudLoginConfig,
    SqliteDataStoreConfig, TracingConfig,
};
use clap::Parser;
use rustical_caldav::CalDavConfig;
use rustical_frontend::FrontendConfig;

mod health;
pub mod membership;
pub mod principals;

pub use health::{HealthArgs, cmd_health};
pub use principals::{PrincipalsArgs, cmd_principals};

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
        },
        oidc: None,
        dav_push: DavPushConfig::default(),
        nextcloud_login: NextcloudLoginConfig::default(),
    };
    let generated_config = toml::to_string(&config)?;
    println!("{generated_config}");
    Ok(())
}
