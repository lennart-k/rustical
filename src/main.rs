use crate::config::Config;
use actix_web::HttpServer;
use anyhow::Result;
use app::make_app;
use clap::Parser;
use config::{CalendarStoreConfig, SqliteCalendarStoreConfig, TomlCalendarStoreConfig};
use rustical_auth::AuthProvider;
use rustical_store::sqlite_store::{create_db_pool, SqliteCalendarStore};
use rustical_store::toml_store::TomlCalendarStore;
use rustical_store::CalendarStore;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

mod app;
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
            let db = create_db_pool(db_url, migrate).await?;
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

    HttpServer::new(move || make_app(cal_store.clone(), auth.clone()))
        .bind((config.http.host, config.http.port))?
        .run()
        .await?;

    Ok(())
}
