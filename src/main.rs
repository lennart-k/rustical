use std::fs;
use std::sync::Arc;

use crate::config::Config;
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use clap::Parser;
use config::{CalendarStoreConfig, TomlCalendarStoreConfig};
use rustical_api::configure_api;
use rustical_dav::{configure_dav, configure_well_known};
use rustical_frontend::configure_frontend;
use rustical_store::calendar::TomlCalendarStore;
use tokio::sync::RwLock;

mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, env)]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let args = Args::parse();
    let config: Config = toml::from_str(&fs::read_to_string(&args.config_file)?)?;

    let cal_store = Arc::new(RwLock::new(match &config.calendar_store {
        CalendarStoreConfig::Toml(TomlCalendarStoreConfig { db_path }) => {
            match fs::read_to_string(db_path) {
                Ok(content) => toml::from_str::<TomlCalendarStore>(&content).unwrap(),
                Err(_) => TomlCalendarStore::new(db_path.to_string()),
            }
        }
    }));

    let auth = match config.auth {
        config::AuthConfig::Htpasswd(config) => 1,
        _ => panic!("invalid auth config"),
    };

    HttpServer::new(move || {
        let cal_store = cal_store.clone();
        App::new()
            .wrap(Logger::new("[%s] %r"))
            .wrap(NormalizePath::trim())
            .service(
                web::scope("/caldav")
                    .configure(|cfg| configure_dav(cfg, "/caldav".to_string(), cal_store.clone())),
            )
            .service(
                web::scope("/.well-known")
                    .configure(|cfg| configure_well_known(cfg, "/caldav".to_string())),
            )
            .service(
                web::scope("/api").configure(|cfg| configure_api(cfg, cal_store.clone().into())),
            )
            .service(
                web::scope("/frontend")
                    .configure(|cfg| configure_frontend(cfg, cal_store.clone().into())),
            )
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await?;
    Ok(())
}
