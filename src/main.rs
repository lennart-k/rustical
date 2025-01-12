use crate::config::Config;
use actix_web::http::{KeepAlive, StatusCode};
use actix_web::HttpServer;
use anyhow::Result;
use app::make_app;
use clap::{Parser, Subcommand};
use commands::{cmd_gen_config, cmd_pwhash};
use config::{DataStoreConfig, SqliteDataStoreConfig};
use rustical_dav::xml::multistatus::PropstatElement;
use rustical_store::auth::StaticUserStore;
use rustical_store::{AddressbookStore, CalendarStore, CollectionOperation, SubscriptionStore};
use rustical_store_sqlite::calendar_store::SqliteCalendarStore;
use rustical_store_sqlite::{create_db_pool, SqliteStore};
use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};
use setup_tracing::setup_tracing;
use std::fs;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use tracing::{error, info};

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
    Pwhash(commands::PwhashArgs),
}

async fn get_data_stores(
    migrate: bool,
    config: &DataStoreConfig,
) -> Result<(
    Arc<dyn AddressbookStore>,
    Arc<dyn CalendarStore>,
    Arc<dyn SubscriptionStore>,
    Receiver<CollectionOperation>,
)> {
    Ok(match &config {
        DataStoreConfig::Sqlite(SqliteDataStoreConfig { db_url }) => {
            let db = create_db_pool(db_url, migrate).await?;
            // Channel to watch for changes (for DAV Push)
            let (send, recv) = tokio::sync::mpsc::channel(1000);

            let addressbook_store = Arc::new(SqliteStore::new(db.clone()));
            let cal_store = Arc::new(SqliteCalendarStore::new(db.clone(), send));
            let subscription_store = Arc::new(SqliteStore::new(db.clone()));
            (addressbook_store, cal_store, subscription_store, recv)
        }
    })
}

// TODO: Move this code somewhere else :)

#[derive(XmlSerialize, Debug)]
struct PushMessageProp {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    topic: String,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    sync_token: Option<String>,
}

#[derive(XmlSerialize, XmlRootTag, Debug)]
#[xml(root = b"push-message", ns = "rustical_dav::namespace::NS_DAVPUSH")]
#[xml(ns_prefix(
    rustical_dav::namespace::NS_DAVPUSH = b"",
    rustical_dav::namespace::NS_DAV = b"D",
))]
struct PushMessage {
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    propstat: PropstatElement<PushMessageProp>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::GenConfig(gen_config_args)) => cmd_gen_config(gen_config_args)?,
        Some(Command::Pwhash(pwhash_args)) => cmd_pwhash(pwhash_args)?,
        None => {
            let config: Config = toml::from_str(&fs::read_to_string(&args.config_file)?)?;

            setup_tracing(&config.tracing);

            let (addr_store, cal_store, subscription_store, mut update_recv) =
                get_data_stores(!args.no_migrations, &config.data_store).await?;

            let subscription_store_clone = subscription_store.clone();
            tokio::spawn(async move {
                let subscription_store = subscription_store_clone.clone();
                while let Some(message) = update_recv.recv().await {
                    dbg!(&message);
                    if let Ok(subscribers) =
                        subscription_store.get_subscriptions(&message.topic).await
                    {
                        let status = match message.r#type {
                            rustical_store::CollectionOperationType::Object => StatusCode::OK,
                            rustical_store::CollectionOperationType::Delete => {
                                StatusCode::NOT_FOUND
                            }
                        };
                        let push_message = PushMessage {
                            propstat: PropstatElement {
                                prop: PushMessageProp {
                                    topic: message.topic,
                                    sync_token: message.sync_token,
                                },
                                status,
                            },
                        };
                        let mut output: Vec<_> =
                            b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n".into();
                        let mut writer = quick_xml::Writer::new_with_indent(&mut output, b' ', 4);
                        if let Err(err) = push_message.serialize_root(&mut writer) {
                            error!("Could not serialize push message: {}", err);
                            continue;
                        }
                        let payload = String::from_utf8(output).unwrap();
                        for subscriber in subscribers {
                            info!(
                                "Sending a push message to {}: {}",
                                subscriber.push_resource, payload
                            );
                            let client = reqwest::Client::new();
                            if let Err(err) = client
                                .post(subscriber.push_resource)
                                .body(payload.to_owned())
                                .send()
                                .await
                            {
                                error!("{err}");
                            }
                        }
                    }
                }
            });

            let user_store = Arc::new(match config.auth {
                config::AuthConfig::Static(config) => StaticUserStore::new(config),
            });

            HttpServer::new(move || {
                make_app(
                    addr_store.clone(),
                    cal_store.clone(),
                    subscription_store.clone(),
                    user_store.clone(),
                    config.frontend.clone(),
                )
            })
            .bind((config.http.host, config.http.port))?
            // Workaround for a weird bug where
            // new requests might timeout since they cannot properly reuse the connection
            // https://github.com/lennart-k/rustical/issues/10
            .keep_alive(KeepAlive::Disabled)
            .run()
            .await?;
        }
    }
    Ok(())
}
