#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
use crate::config::{Config, HttpBindConfig};
use anyhow::{Result, anyhow};
use app::make_app;
use axum::ServiceExt;
use axum::extract::Request;
use clap::{Parser, Subcommand};
use config::{DataStoreConfig, PostgresDataStoreConfig, SqliteDataStoreConfig};
use provided_listeners::ProvidedListeners;
use rustical_dav_push::{DavPushController, DavPushStore};
use rustical_store::auth::AuthenticationProvider;
use rustical_store::{AddressbookStore, CalendarStore, CollectionOperation, PrefixedCalendarStore};
use rustical_store_postgres::addressbook_store::PostgresAddressbookStore;
use rustical_store_postgres::calendar_store::PostgresCalendarStore;
use rustical_store_postgres::principal_store::PostgresPrincipalStore;
use rustical_store_postgres::{PostgresStore, create_db_pool as create_postgres_pool};
use rustical_store_sqlite::addressbook_store::SqliteAddressbookStore;
use rustical_store_sqlite::calendar_store::SqliteCalendarStore;
use rustical_store_sqlite::principal_store::SqlitePrincipalStore;
use rustical_store_sqlite::{SqliteStore, create_db_pool};
use setup_tracing::setup_tracing;
use std::fs;
use std::os::unix::fs::FileTypeExt;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::sync::mpsc::Receiver;
use tower::Layer;
use tower_http::normalize_path::NormalizePathLayer;
use tracing::{info, warn};

pub mod app;
mod commands;
mod tasks;
pub use commands::*;
pub mod config;
mod setup_tracing;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, env, default_value = "/etc/rustical/config.toml")]
    pub config_file: String,
    #[arg(long, env, help = "Do no run database migrations (only for sql store)")]
    pub no_migrations: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Run the RustiCal server")]
    Serve,
    GenConfig(commands::GenConfigArgs),
    #[command(
        about = "Healthcheck for running instance (Used for HEALTHCHECK in Docker container)"
    )]
    Health(HealthArgs),
    Principals(PrincipalsArgs),
}

#[allow(clippy::missing_errors_doc)]
pub(crate) async fn get_sqlite_data_stores(
    migrate: bool,
    SqliteDataStoreConfig {
        db_url,
        run_repairs,
        skip_broken,
    }: &SqliteDataStoreConfig,
) -> Result<(
    Arc<SqliteAddressbookStore>,
    Arc<SqliteCalendarStore>,
    Arc<SqliteStore>,
    Arc<SqlitePrincipalStore>,
    Receiver<CollectionOperation>,
)> {
    let db = create_db_pool(db_url, migrate).await?;
    let (send, recv) = tokio::sync::mpsc::channel(1000);
    let addressbook_store = Arc::new(SqliteAddressbookStore::new(
        db.clone(),
        send.clone(),
        *skip_broken,
    ));
    let cal_store = Arc::new(SqliteCalendarStore::new(db.clone(), send, *skip_broken));
    if *run_repairs {
        info!("Running repair tasks");
        addressbook_store.repair_orphans().await?;
        cal_store.repair_invalid_version_4_0().await?;
        cal_store.repair_orphans().await?;
    }
    let subscription_store = Arc::new(SqliteStore::new(db.clone()));
    let principal_store = Arc::new(SqlitePrincipalStore::new(db));
    for principal in principal_store.get_principals().await? {
        cal_store.validate_objects(&principal.id).await?;
        addressbook_store.validate_objects(&principal.id).await?;
    }
    Ok((
        addressbook_store,
        cal_store,
        subscription_store,
        principal_store,
        recv,
    ))
}

#[allow(clippy::missing_errors_doc)]
pub(crate) async fn get_postgres_data_stores(
    migrate: bool,
    PostgresDataStoreConfig {
        db_url,
        run_repairs,
        skip_broken,
    }: &PostgresDataStoreConfig,
) -> Result<(
    Arc<PostgresAddressbookStore>,
    Arc<PostgresCalendarStore>,
    Arc<PostgresStore>,
    Arc<PostgresPrincipalStore>,
    Receiver<CollectionOperation>,
)> {
    let db = create_postgres_pool(db_url, migrate).await?;
    let (send, recv) = tokio::sync::mpsc::channel(1000);
    let addressbook_store = Arc::new(PostgresAddressbookStore::new(
        db.clone(),
        send.clone(),
        *skip_broken,
    ));
    let cal_store = Arc::new(PostgresCalendarStore::new(db.clone(), send, *skip_broken));
    if *run_repairs {
        info!("Running repair tasks");
        addressbook_store.repair_orphans().await?;
        cal_store.repair_invalid_version_4_0().await?;
        cal_store.repair_orphans().await?;
    }
    let subscription_store = Arc::new(PostgresStore::new(db.clone()));
    let principal_store = Arc::new(PostgresPrincipalStore::new(db));
    for principal in principal_store.get_principals().await? {
        cal_store.validate_objects(&principal.id).await?;
        addressbook_store.validate_objects(&principal.id).await?;
    }
    Ok((
        addressbook_store,
        cal_store,
        subscription_store,
        principal_store,
        recv,
    ))
}

#[allow(clippy::missing_errors_doc, clippy::missing_panics_doc)]
pub async fn cmd_serve(
    args: Args,
    config: Config,
    start_notifier: Option<Arc<Notify>>,
    tracing: bool,
) -> Result<()> {
    if tracing {
        setup_tracing(&config.tracing);
    }

    match &config.data_store {
        DataStoreConfig::Sqlite(cfg) => {
            let (addr_store, cal_store, subscription_store, principal_store, update_recv) =
                get_sqlite_data_stores(!args.no_migrations, cfg).await?;
            serve_with_stores(
                config,
                start_notifier,
                addr_store,
                cal_store,
                subscription_store,
                principal_store,
                update_recv,
            )
            .await
        }
        DataStoreConfig::Postgres(cfg) => {
            let (addr_store, cal_store, subscription_store, principal_store, update_recv) =
                get_postgres_data_stores(!args.no_migrations, cfg).await?;
            serve_with_stores(
                config,
                start_notifier,
                addr_store,
                cal_store,
                subscription_store,
                principal_store,
                update_recv,
            )
            .await
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]
async fn serve_with_stores<
    AS: AddressbookStore + PrefixedCalendarStore + 'static,
    CS: CalendarStore + 'static,
    DP: DavPushStore + 'static,
    AP: AuthenticationProvider + 'static,
>(
    config: Config,
    start_notifier: Option<Arc<Notify>>,
    addr_store: Arc<AS>,
    cal_store: Arc<CS>,
    subscription_store: Arc<DP>,
    principal_store: Arc<AP>,
    update_recv: Receiver<CollectionOperation>,
) -> Result<()> {
    if config.dav_push.enabled {
        let dav_push_controller = DavPushController::new(
            config.dav_push.allowed_push_servers,
            subscription_store.clone(),
        );
        // Atm we never join this task
        tokio::spawn(async move {
            dav_push_controller.notifier(update_recv).await;
        });
    }

    let app = make_app(
        addr_store.clone(),
        cal_store.clone(),
        subscription_store.clone(),
        principal_store.clone(),
        config.frontend.clone(),
        config.oidc.clone(),
        config.caldav,
        &config.nextcloud_login,
        config.dav_push.enabled,
        config.http.session_cookie_samesite_strict,
        config.http.payload_limit_mb,
    );
    let app = ServiceExt::<Request>::into_make_service(
        NormalizePathLayer::trim_trailing_slash().layer(app),
    );

    let mut provided_listeners = ProvidedListeners::from_env()?;
    if let Some(trash_retention_days) = config.maintenance.trash_retention_days {
        tokio::spawn(tasks::cleanup_trashed_calendar_entities(
            cal_store.clone(),
            trash_retention_days,
            shutdown_signal(),
        ));
    }

    let bind_config = config.http.bind_config()?;
    let serve_task = match bind_config {
        HttpBindConfig::Tcp(address) => {
            let listener = provided_listeners
                .tcp_tokio_resolved_or_bind(&address)
                .await?;

            tokio::spawn(async move {
                info!("RustiCal serving on http://{address}");
                if let Some(start_notifier) = start_notifier {
                    start_notifier.notify_waiters();
                }
                axum::serve(listener, app)
                    .with_graceful_shutdown(shutdown_signal())
                    .await
                    .unwrap();
            })
        }

        HttpBindConfig::Unix(path) => {
            let listener = if let Some(listener) = provided_listeners.unix_tokio(path.as_path()) {
                listener?
            } else {
                if path.exists() {
                    let metadata = fs::metadata(&path)?;
                    if metadata.file_type().is_socket() {
                        // Only remove existing file if it's a socket
                        fs::remove_file(&path)?;
                    } else {
                        return Err(anyhow!(
                            "Path {path} exists and is not a socket",
                            path = path.display()
                        ));
                    }
                }

                tokio::net::UnixListener::bind(&path)?
            };

            tokio::spawn(async move {
                info!("RustiCal serving on unix://{path}", path = path.display());
                if let Some(start_notifier) = start_notifier {
                    start_notifier.notify_waiters();
                }
                axum::serve(listener, app)
                    .with_graceful_shutdown(shutdown_signal())
                    .await
                    .unwrap();
            })
        }
    };

    serve_task.await?;

    Ok(())
}

async fn shutdown_signal() -> () {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::debug!("Received SIGINT signal"),
        () = terminate => tracing::debug!("Received SIGTERM signal"),
    }
}
