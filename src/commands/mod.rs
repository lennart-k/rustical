use crate::config::{
    Config, DataStoreConfig, DavPushConfig, HttpConfig, MaintenanceConfig, NextcloudLoginConfig,
    PostgresDataStoreConfig, SqliteDataStoreConfig, TracingConfig,
};
use clap::Parser;
use rustical_caldav::CalDavConfig;
use rustical_frontend::FrontendConfig;

pub mod app_token;
mod health;
pub mod membership;
pub mod principals;

pub use health::{HealthArgs, cmd_health};
pub use principals::{PrincipalsArgs, cmd_principals};

#[derive(Debug, Parser, Clone, Copy)]
pub struct GenConfigArgs {
    /// Emit a `PostgreSQL` data store instead of `SQLite`
    #[arg(long)]
    pub postgres: bool,
}

fn sample_config(postgres: bool) -> Config {
    Config {
        http: HttpConfig::default(),
        caldav: CalDavConfig::default(),
        data_store: if postgres {
            DataStoreConfig::Postgres(PostgresDataStoreConfig {
                db_url: "postgres://rustical@localhost/rustical".to_owned(),
                run_repairs: true,
                skip_broken: true,
            })
        } else {
            DataStoreConfig::Sqlite(SqliteDataStoreConfig {
                db_url: "/var/lib/rustical/db.sqlite3".to_owned(),
                run_repairs: true,
                skip_broken: true,
            })
        },
        tracing: TracingConfig::default(),
        frontend: FrontendConfig {
            enabled: true,
            allow_password_login: true,
        },
        oidc: None,
        dav_push: DavPushConfig::default(),
        nextcloud_login: NextcloudLoginConfig::default(),
        maintenance: MaintenanceConfig::default(),
    }
}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub fn cmd_gen_config(args: GenConfigArgs) -> anyhow::Result<()> {
    println!("{}", toml::to_string(&sample_config(args.postgres))?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_config_postgres_emits_postgres_store() {
        let config = sample_config(true);
        match config.data_store.clone() {
            DataStoreConfig::Postgres(pg) => {
                assert!(pg.db_url.starts_with("postgres://"));
                assert!(pg.run_repairs);
                assert!(pg.skip_broken);
            }
            DataStoreConfig::Sqlite(_) => panic!("expected postgres"),
        }
        assert!(
            toml::to_string(&config)
                .unwrap()
                .contains("backend = \"postgres\"")
        );
    }
}
