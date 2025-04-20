use argon2::password_hash::SaltString;
use clap::{Parser, ValueEnum};
use password_hash::PasswordHasher;
use pbkdf2::Params;
use rand::{RngCore, rngs::OsRng};
use rustical_frontend::FrontendConfig;
use rustical_store::auth::TomlUserStoreConfig;

use crate::config::{
    AuthConfig, Config, DataStoreConfig, DavPushConfig, HttpConfig, SqliteDataStoreConfig,
    TracingConfig,
};

pub mod principals;

#[derive(Debug, Parser)]
pub struct GenConfigArgs {}

pub fn generate_frontend_secret() -> [u8; 64] {
    let mut rng = rand::thread_rng();

    let mut secret = [0u8; 64];
    rng.fill_bytes(&mut secret);
    secret
}

pub fn cmd_gen_config(_args: GenConfigArgs) -> anyhow::Result<()> {
    let config = Config {
        http: HttpConfig::default(),
        auth: AuthConfig::Toml(TomlUserStoreConfig {
            path: "/etc/rustical/principals.toml".to_owned(),
        }),
        data_store: DataStoreConfig::Sqlite(SqliteDataStoreConfig {
            db_url: "".to_owned(),
        }),
        tracing: TracingConfig::default(),
        frontend: FrontendConfig {
            secret_key: generate_frontend_secret(),
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
        default_value_t = 1000
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
