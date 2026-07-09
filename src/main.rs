#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use anyhow::Result;
use clap::Parser;
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use rustical::config::Config;
use rustical::env_file::EnvFile;
use rustical::{Args, Command};
use rustical::{cmd_default, cmd_gen_config, cmd_health, cmd_principals};
use tracing::warn;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let parse_config = || {
        Figment::new()
            .merge(Toml::file(&args.config_file))
            .merge(
                Env::prefixed("RUSTICAL_")
                    // *_FILE variables are handled by the EnvFile provider below
                    .filter(|key| !key.as_str().to_ascii_lowercase().ends_with("_file"))
                    .split("__"),
            )
            .merge(EnvFile::prefixed("RUSTICAL_").split("__"))
            .extract()
            // Clippy appeasement clippy::result_large_err
            .map_err(anyhow::Error::from)
    };

    match args.command {
        Some(Command::GenConfig(gen_config_args)) => cmd_gen_config(gen_config_args),
        Some(Command::Principals(principals_args)) => {
            cmd_principals(principals_args, parse_config()?).await
        }
        Some(Command::Health(health_args)) => {
            let config: Config = parse_config()?;
            cmd_health(config.http, health_args).await
        }
        None => {
            let config: Config = parse_config()?;
            cmd_default(args, config, None, true).await
        }
    }
}

#[cfg(test)]
mod test_config {
    use figment::{
        Figment, Jail,
        providers::{Env, Format, Toml},
    };
    use rustical::config::{Config, HttpBindConfig};

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
}
