#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use crate::commands::health::{HealthArgs, cmd_health};
use crate::config::Config;
use anyhow::Result;
use app::make_app;
use axum::ServiceExt;
use axum::extract::Request;
use clap::{Parser, Subcommand};
use commands::cmd_gen_config;
use commands::principals::{PrincipalsArgs, cmd_principals};
use config::{DataStoreConfig, SqliteDataStoreConfig};
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use rustical_dav_push::DavPushController;
use rustical_store::auth::AuthenticationProvider;
use rustical_store::{
    AddressbookStore, CalendarStore, CollectionOperation, PrefixedCalendarStore, SubscriptionStore,
};
use rustical_store_sqlite::addressbook_store::SqliteAddressbookStore;
use rustical_store_sqlite::calendar_store::SqliteCalendarStore;
use rustical_store_sqlite::principal_store::SqlitePrincipalStore;
use rustical_store_sqlite::{SqliteStore, create_db_pool};
use setup_tracing::setup_tracing;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;
use tracing::info;

mod app;
mod commands;
mod config;
mod setup_tracing;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, env, default_value = "/etc/rustical/config.toml")]
    config_file: String,
    #[arg(long, env, help = "Do no run database migrations (only for sql store)")]
    no_migrations: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    GenConfig(commands::GenConfigArgs),
    Principals(PrincipalsArgs),
    #[command(
        about = "Healthcheck for running instance (Used for HEALTHCHECK in Docker container)"
    )]
    Health(HealthArgs),
}

async fn get_data_stores(
    migrate: bool,
    config: &DataStoreConfig,
) -> Result<(
    Arc<impl AddressbookStore + PrefixedCalendarStore>,
    Arc<impl CalendarStore>,
    Arc<impl SubscriptionStore>,
    Arc<impl AuthenticationProvider>,
    Receiver<CollectionOperation>,
)> {
    Ok(match &config {
        DataStoreConfig::Sqlite(SqliteDataStoreConfig { db_url }) => {
            let db = create_db_pool(db_url, migrate).await?;
            // Channel to watch for changes (for DAV Push)
            let (send, recv) = tokio::sync::mpsc::channel(1000);

            let addressbook_store = Arc::new(SqliteAddressbookStore::new(db.clone(), send.clone()));
            addressbook_store.repair_orphans().await?;
            let cal_store = Arc::new(SqliteCalendarStore::new(db.clone(), send));
            cal_store.repair_orphans().await?;
            let subscription_store = Arc::new(SqliteStore::new(db.clone()));
            let principal_store = Arc::new(SqlitePrincipalStore::new(db));
            (
                addressbook_store,
                cal_store,
                subscription_store,
                principal_store,
                recv,
            )
        }
    })
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::GenConfig(gen_config_args)) => cmd_gen_config(gen_config_args)?,
        Some(Command::Principals(principals_args)) => cmd_principals(principals_args).await?,
        Some(Command::Health(health_args)) => {
            let config: Config = Figment::new()
                .merge(Toml::file(&args.config_file))
                .merge(Env::prefixed("RUSTICAL_").split("__"))
                .extract()?;
            cmd_health(config.http, health_args).await?;
        }
        None => {
            let config: Config = Figment::new()
                .merge(Toml::file(&args.config_file))
                .merge(Env::prefixed("RUSTICAL_").split("__"))
                .extract()?;

            setup_tracing(&config.tracing);

            let (addr_store, cal_store, subscription_store, principal_store, update_recv) =
                get_data_stores(!args.no_migrations, &config.data_store).await?;

            let mut tasks = vec![];

            if config.dav_push.enabled {
                let dav_push_controller = DavPushController::new(
                    config.dav_push.allowed_push_servers,
                    subscription_store.clone(),
                );
                tasks.push(tokio::spawn(async move {
                    dav_push_controller.notifier(update_recv).await;
                }));
            }

            let app = make_app(
                addr_store.clone(),
                cal_store.clone(),
                subscription_store.clone(),
                principal_store.clone(),
                config.frontend.clone(),
                config.oidc.clone(),
                &config.nextcloud_login,
                config.dav_push.enabled,
                config.http.session_cookie_samesite_strict,
                config.http.payload_limit_mb,
            );
            let app = ServiceExt::<Request>::into_make_service(
                NormalizePathLayer::trim_trailing_slash().layer(app),
            );

            let address = format!("{}:{}", config.http.host, config.http.port);
            let listener = tokio::net::TcpListener::bind(&address).await?;
            tasks.push(tokio::spawn(async move {
                info!("RustiCal serving on http://{address}");
                axum::serve(listener, app).await.unwrap();
            }));

            for task in tasks {
                task.await?;
            }
        }
    }
    Ok(())
}
