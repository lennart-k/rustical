#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use anyhow::Result;
use clap::Parser;
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use rustical::config::Config;
use rustical::{Args, Command};
use rustical::{cmd_gen_config, cmd_health, cmd_principals, cmd_serve};
use tracing::warn;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let parse_config = || {
        Figment::new()
            .merge(Toml::file(&args.config_file))
            .merge(Env::prefixed("RUSTICAL_").split("__"))
            .extract()
            // Clippy appeasement clippy::result_large_err
            .map_err(anyhow::Error::from)
    };

    match args.command {
        Command::GenConfig(gen_config_args) => cmd_gen_config(gen_config_args),
        Command::Principals(principals_args) => {
            cmd_principals(principals_args, parse_config()?).await
        }
        Command::Health(health_args) => {
            let config: Config = parse_config()?;
            cmd_health(config.http, health_args).await
        }
        Command::Serve => {
            let config: Config = parse_config()?;
            cmd_serve(args, config, None, true).await
        }
    }
}

#[cfg(test)]
mod test_config {
    use figment::{
        Figment, Jail,
        providers::{Env, Format, Toml},
    };
    use rustical::config::{Config, DataStoreConfig, HttpBindConfig};

    #[test]
    fn test_config_toml_http_host() {
        let config = r#"
[data_store.sqlite]
db_url = "/var/lib/rustical/db.sqlite3"

[http]
host = "0.0.0.0"
port = 4000

[oidc]
name = "Authelia"
issuer = "https://auth.rustical.dev"
client_id = "rustical"
client_secret = "secret"
claim_userid = "email"
scopes = ["openid", "email", "profile", "groups"]
require_group = "app:rustical"
allow_sign_up = true
"#;

        let config: Config = Figment::new()
            .merge(Toml::string(config))
            .extract()
            .unwrap();
        assert_eq!(
            config.http.bind_config().unwrap(),
            HttpBindConfig::Tcp("0.0.0.0:4000".to_string())
        );
    }

    #[test]
    fn test_config_env_http_host() {
        Jail::expect_with(|jail| {
            jail.set_env(
                "RUSTICAL_DATA_STORE__SQLITE__DB_URL",
                "/var/lib/rustical/db.sqlite3",
            );
            jail.set_env("RUSTICAL_HTTP__HOST", "localhost");
            jail.set_env("RUSTICAL_HTTP__PORT", "4000");

            let config: Config = Figment::new()
                .merge(Env::prefixed("RUSTICAL_").split("__"))
                .extract()
                .unwrap();
            assert_eq!(
                config.http.bind_config().unwrap(),
                HttpBindConfig::Tcp("localhost:4000".to_string())
            );
            Ok(())
        });
    }

    #[test]
    fn test_config_toml_http_bind() {
        let config = r#"
[data_store.sqlite]
db_url = "/var/lib/rustical/db.sqlite3"

[http]
bind = "0.0.0.0:4000"

[oidc]
name = "Authelia"
issuer = "https://auth.rustical.dev"
client_id = "rustical"
client_secret = "secret"
claim_userid = "email"
scopes = ["openid", "email", "profile", "groups"]
require_group = "app:rustical"
allow_sign_up = true
"#;

        let config: Config = Figment::new()
            .merge(Toml::string(config))
            .extract()
            .unwrap();
        assert_eq!(
            config.http.bind_config().unwrap(),
            HttpBindConfig::Tcp("0.0.0.0:4000".to_string())
        );
    }

    #[test]
    fn test_config_env_http_bind() {
        Jail::expect_with(|jail| {
            jail.set_env(
                "RUSTICAL_DATA_STORE__SQLITE__DB_URL",
                "/var/lib/rustical/db.sqlite3",
            );
            jail.set_env("RUSTICAL_HTTP__BIND", "localhost:4000");

            let config: Config = Figment::new()
                .merge(Env::prefixed("RUSTICAL_").split("__"))
                .extract()
                .unwrap();
            assert_eq!(
                config.http.bind_config().unwrap(),
                HttpBindConfig::Tcp("localhost:4000".to_string())
            );
            Ok(())
        });
    }

    #[test]
    fn test_config_env_http_unix() {
        Jail::expect_with(|jail| {
            jail.set_env(
                "RUSTICAL_DATA_STORE__SQLITE__DB_URL",
                "/var/lib/rustical/db.sqlite3",
            );
            jail.set_env("RUSTICAL_HTTP__BIND", "unix:/run/rustical/socket");

            let config: Config = Figment::new()
                .merge(Env::prefixed("RUSTICAL_").split("__"))
                .extract()
                .unwrap();
            assert_eq!(
                config.http.bind_config().unwrap(),
                HttpBindConfig::Unix("/run/rustical/socket".parse().unwrap())
            );
            Ok(())
        });
    }

    #[test]
    fn test_config_toml_postgres_db_url() {
        let config: Config = Figment::new()
            .merge(Toml::string(
                r#"
[data_store.postgres]
db_url = "postgres://user@localhost/rustical"
"#,
            ))
            .extract()
            .unwrap();
        match config.data_store {
            DataStoreConfig::Postgres(pg) => {
                assert_eq!(pg.db_url, "postgres://user@localhost/rustical");
                assert!(pg.run_repairs);
                assert!(pg.skip_broken);
            }
            DataStoreConfig::Sqlite(_) => panic!("expected postgres"),
        }
    }

    #[test]
    fn test_config_env_postgres_db_url() {
        Jail::expect_with(|jail| {
            jail.set_env(
                "RUSTICAL_DATA_STORE__POSTGRES__DB_URL",
                "postgres://user@localhost/rustical",
            );

            let config: Config = Figment::new()
                .merge(Env::prefixed("RUSTICAL_").split("__"))
                .extract()
                .unwrap();
            match config.data_store {
                DataStoreConfig::Postgres(pg) => {
                    assert_eq!(pg.db_url, "postgres://user@localhost/rustical");
                }
                DataStoreConfig::Sqlite(_) => panic!("expected postgres"),
            }
            Ok(())
        });
    }

    #[test]
    fn test_config_env_backend_selects_postgres_over_docker_sqlite_default() {
        Jail::expect_with(|jail| {
            jail.set_env("RUSTICAL_DATA_STORE__BACKEND", "postgres");
            jail.set_env(
                "RUSTICAL_DATA_STORE__SQLITE__DB_URL",
                "/var/lib/rustical/db.sqlite3",
            );
            jail.set_env(
                "RUSTICAL_DATA_STORE__POSTGRES__DB_URL",
                "postgres://user@localhost/rustical",
            );

            let config: Config = Figment::new()
                .merge(Env::prefixed("RUSTICAL_").split("__"))
                .extract()
                .unwrap();
            assert!(matches!(config.data_store, DataStoreConfig::Postgres(_)));
            Ok(())
        });
    }

    #[test]
    fn test_config_env_requires_backend_for_two_stores() {
        Jail::expect_with(|jail| {
            jail.set_env(
                "RUSTICAL_DATA_STORE__SQLITE__DB_URL",
                "/var/lib/rustical/db.sqlite3",
            );
            jail.set_env(
                "RUSTICAL_DATA_STORE__POSTGRES__DB_URL",
                "postgres://user@localhost/rustical",
            );

            let config = Figment::new()
                .merge(Env::prefixed("RUSTICAL_").split("__"))
                .extract::<Config>();
            assert!(config.is_err());
            Ok(())
        });
    }
}
