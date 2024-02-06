use crate::config::Config;
use actix_web::middleware::{Logger, NormalizePath};
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use clap::Parser;
use config::{CalendarStoreConfig, SqliteCalendarStoreConfig, TomlCalendarStoreConfig};
use rustical_api::configure_api;
use rustical_auth::AuthProvider;
use rustical_caldav::{configure_dav, configure_well_known};
use rustical_store::calendar::CalendarStore;
use rustical_store::sqlite_store::SqliteCalendarStore;
use rustical_store::toml_store::TomlCalendarStore;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::SqlitePool;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

mod config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, env)]
    config_file: String,
    #[arg(long, env, help = "Run database migrations (only for sql store)")]
    migrate: bool,
}

async fn get_cal_store(
    migrate: bool,
    config: &CalendarStoreConfig,
) -> Result<Arc<RwLock<dyn CalendarStore>>> {
    let cal_store: Arc<RwLock<dyn CalendarStore>> = match &config {
        CalendarStoreConfig::Toml(TomlCalendarStoreConfig { db_path }) => {
            Arc::new(RwLock::new(match fs::read_to_string(db_path) {
                Ok(content) => toml::from_str::<TomlCalendarStore>(&content).unwrap(),
                Err(_) => TomlCalendarStore::new(db_path.to_string()),
            }))
        }
        CalendarStoreConfig::Sqlite(SqliteCalendarStoreConfig { db_url }) => {
            let db = SqlitePool::connect_with(
                SqliteConnectOptions::new()
                    .filename(db_url)
                    .create_if_missing(true),
            )
            .await?;
            if migrate {
                println!("Running database migrations");
                sqlx::migrate!("./migrations").run(&db).await?;
            }
            Arc::new(RwLock::new(SqliteCalendarStore::new(db)))
        }
    };
    Ok(cal_store)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let args = Args::parse();
    let config: Config = toml::from_str(&fs::read_to_string(&args.config_file)?)?;

    let cal_store = get_cal_store(args.migrate, &config.calendar_store).await?;

    let auth: Arc<AuthProvider> = Arc::new(config.auth.into());

    HttpServer::new(move || {
        let cal_store = cal_store.clone();
        App::new()
            .wrap(Logger::new("[%s] %r"))
            .wrap(NormalizePath::trim())
            .service(web::scope("/caldav").configure(|cfg| {
                configure_dav(cfg, "/caldav".to_string(), auth.clone(), cal_store.clone())
            }))
            .service(
                web::scope("/.well-known")
                    .configure(|cfg| configure_well_known(cfg, "/caldav".to_string())),
            )
            .service(
                web::scope("/api").configure(|cfg| configure_api(cfg, cal_store.clone().into())),
            )
    })
    .bind((config.http.host, config.http.port))?
    .run()
    .await?;
    Ok(())
}
