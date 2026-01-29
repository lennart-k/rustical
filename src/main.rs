#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use anyhow::Result;
use clap::Parser;
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use rustical::config::Config;
use rustical::{Args, Command};
use rustical::{cmd_default, cmd_gen_config, cmd_health, cmd_principals};
use tracing::warn;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let parse_config = || {
        Figment::new()
            .merge(Toml::file(&args.config_file))
            .merge(Env::prefixed("RUSTICAL_").split("__"))
            .extract()
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
