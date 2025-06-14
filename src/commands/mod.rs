use argon2::password_hash::SaltString;
use clap::{Parser, ValueEnum};
use password_hash::{PasswordHasher, rand_core::OsRng};
use pbkdf2::Params;
use rustical_frontend::FrontendConfig;

use crate::config::{
    Config, DataStoreConfig, DavPushConfig, HttpConfig, SqliteDataStoreConfig, TracingConfig,
};

mod membership;
pub mod principals;

#[derive(Debug, Parser)]
pub struct GenConfigArgs {}

pub fn cmd_gen_config(_args: GenConfigArgs) -> anyhow::Result<()> {
    let config = Config {
        http: HttpConfig::default(),
        data_store: DataStoreConfig::Sqlite(SqliteDataStoreConfig {
            db_url: "/var/lib/rustical/db.sqlite3".to_owned(),
        }),
        tracing: TracingConfig::default(),
        frontend: FrontendConfig {
            enabled: true,
            allow_password_login: true,
        },
        oidc: None,
        dav_push: DavPushConfig::default(),
        nextcloud_login: Default::default(),
    };
    let generated_config = toml::to_string(&config)?;
    println!("{generated_config}");
    Ok(())
}

#[derive(Debug, Clone, ValueEnum)]
enum PwhashAlgorithm {
    #[value(help = "Use this for your password")]
    Argon2,
    #[value(help = "Significantly faster algorithm, use for app tokens")]
    Pbkdf2,
}

#[derive(Debug, Parser)]
pub struct PwhashArgs {
    #[arg(long, short = 'a')]
    algorithm: PwhashAlgorithm,
    #[arg(
        long,
        short = 'r',
        help = "ONLY for pbkdf2: Number of rounds to calculate",
        default_value_t = 100
    )]
    rounds: u32,
}

pub fn cmd_pwhash(args: PwhashArgs) -> anyhow::Result<()> {
    println!("Enter your password:");
    let password = rpassword::read_password()?;
    let salt = SaltString::generate(OsRng);
    let password_hash = match args.algorithm {
        PwhashAlgorithm::Argon2 => argon2::Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .unwrap(),
        PwhashAlgorithm::Pbkdf2 => pbkdf2::Pbkdf2
            .hash_password_customized(
                password.as_bytes(),
                None,
                None,
                Params {
                    rounds: args.rounds,
                    ..Default::default()
                },
                &salt,
            )
            .unwrap(),
    };
    println!("{password_hash}");
    Ok(())
}
