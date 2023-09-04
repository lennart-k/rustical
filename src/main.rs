use std::fs;
use std::sync::Arc;

use crate::config::Config;
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use clap::Parser;
use config::{CalendarStoreConfig, JsonCalendarStoreConfig};
use rustical_api::configure_api;
use rustical_dav::{configure_dav, configure_well_known};
use rustical_store::calendar::JsonCalendarStore;
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
    // TODO: Clean this jank up as soon more configuration options appear
    let db_path = match config.calendar_store {
        CalendarStoreConfig::Json(JsonCalendarStoreConfig { db_path }) => db_path,
    };

    let cal_store = Arc::new(RwLock::new(
        if let Ok(json) = fs::read_to_string(&db_path) {
            serde_json::from_str::<JsonCalendarStore>(&json)?
        } else {
            JsonCalendarStore::new(db_path.to_string())
        },
    ));

    HttpServer::new(move || {
        let cal_store = cal_store.clone();
        App::new()
            .wrap(Logger::new("[%s] %r"))
            .wrap(NormalizePath::trim())
            .service(
                web::scope("/dav").configure(|cfg| configure_dav(cfg, cal_store.clone().into())),
            )
            .service(
                web::scope("/.well-known")
                    .configure(|cfg| configure_well_known(cfg, "/dav".to_string())),
            )
            .service(
                web::scope("/api").configure(|cfg| configure_api(cfg, cal_store.clone().into())),
            )
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await?;
    Ok(())
}
