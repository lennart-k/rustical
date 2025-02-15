use crate::config::Config;
use actix_web::http::KeepAlive;
use actix_web::HttpServer;
use anyhow::Result;
use app::make_app;
use axum::Router;
use clap::{Parser, Subcommand};
use commands::{cmd_gen_config, cmd_pwhash};
use config::{DataStoreConfig, SqliteDataStoreConfig};
use figment::providers::{Env, Format, Toml};
use figment::Figment;
use rustical_dav::push::push_notifier;
use rustical_nextcloud_login::NextcloudFlows;
use rustical_store::auth::TomlPrincipalStore;
use rustical_store::{AddressbookStore, CalendarStore, CollectionOperation, SubscriptionStore};
use rustical_store_sqlite::addressbook_store::SqliteAddressbookStore;
use rustical_store_sqlite::calendar_store::SqliteCalendarStore;
use rustical_store_sqlite::{create_db_pool, SqliteStore};
use setup_tracing::setup_tracing;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

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
    Arc<impl AddressbookStore>,
    Arc<impl CalendarStore>,
    Arc<impl SubscriptionStore>,
    Receiver<CollectionOperation>,
)> {
    Ok(match &config {
        DataStoreConfig::Sqlite(SqliteDataStoreConfig { db_url }) => {
            let db = create_db_pool(db_url, migrate).await?;
            // Channel to watch for changes (for DAV Push)
            let (send, recv) = tokio::sync::mpsc::channel(1000);

            let addressbook_store = Arc::new(SqliteAddressbookStore::new(db.clone(), send.clone()));
            let cal_store = Arc::new(SqliteCalendarStore::new(db.clone(), send));
            let subscription_store = Arc::new(SqliteStore::new(db.clone()));
            (addressbook_store, cal_store, subscription_store, recv)
        }
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Command::GenConfig(gen_config_args)) => cmd_gen_config(gen_config_args)?,
        Some(Command::Pwhash(pwhash_args)) => cmd_pwhash(pwhash_args)?,
        None => {
            let config: Config = Figment::new()
                // TODO: What to do when config file does not exist?
                .merge(Toml::file(&args.config_file))
                .merge(Env::prefixed("RUSTICAL_").split("__"))
                .extract()?;

            setup_tracing(&config.tracing);

            let (addr_store, cal_store, subscription_store, update_recv) =
                get_data_stores(!args.no_migrations, &config.data_store).await?;

            if config.dav_push.enabled {
                tokio::spawn(push_notifier(
                    config.dav_push.allowed_push_servers,
                    update_recv,
                    subscription_store.clone(),
                ));
            }

            let user_store = match config.auth {
                config::AuthConfig::Toml(config) => Arc::new(TomlPrincipalStore::new(config)?),
            };

            let nextcloud_flows = Arc::new(NextcloudFlows::default());

            let app = make_app(addr_store, cal_store, subscription_store, user_store);
            let listener =
                tokio::net::TcpListener::bind(format!("{}:{}", config.http.host, config.http.port))
                    .await?;
            axum::serve(listener, app).await?
        }
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use crate::{
//         app::make_app, commands::generate_frontend_secret, config::NextcloudLoginConfig,
//         get_data_stores,
//     };
//     use actix_web::{http::StatusCode, test::TestRequest};
//     use anyhow::anyhow;
//     use async_trait::async_trait;
//     use rustical_frontend::FrontendConfig;
//     use rustical_nextcloud_login::NextcloudFlows;
//     use rustical_store::auth::AuthenticationProvider;
//     use std::sync::Arc;
//
//     #[derive(Debug, Clone)]
//     struct MockUserStore;
//
//     #[async_trait]
//     impl AuthenticationProvider for MockUserStore {
//         async fn get_principal(
//             &self,
//             id: &str,
//         ) -> Result<Option<rustical_store::auth::User>, rustical_store::Error> {
//             Err(rustical_store::Error::NotFound)
//         }
//
//         async fn validate_user_token(
//             &self,
//             user_id: &str,
//             token: &str,
//         ) -> Result<Option<rustical_store::auth::User>, rustical_store::Error> {
//             Err(rustical_store::Error::NotFound)
//         }
//
//         async fn add_app_token(
//             &self,
//             user_id: &str,
//             name: String,
//             token: String,
//         ) -> Result<(), rustical_store::Error> {
//             Err(rustical_store::Error::Other(anyhow!("Not implemented")))
//         }
//     }
//
//     #[tokio::test]
//     async fn test_main() {
//         let (addr_store, cal_store, subscription_store, _update_recv) = get_data_stores(
//             true,
//             &crate::config::DataStoreConfig::Sqlite(crate::config::SqliteDataStoreConfig {
//                 db_url: "".to_owned(),
//             }),
//         )
//         .await
//         .unwrap();
//
//         let user_store = Arc::new(MockUserStore);
//
//         let app = make_app(
//             addr_store,
//             cal_store,
//             subscription_store,
//             user_store,
//             FrontendConfig {
//                 enabled: false,
//                 secret_key: generate_frontend_secret(),
//             },
//             NextcloudLoginConfig { enabled: false },
//             Arc::new(NextcloudFlows::default()),
//         );
//         let app = actix_web::test::init_service(app).await;
//         let req = TestRequest::get().uri("/").to_request();
//         let resp = actix_web::test::call_service(&app, req).await;
//         assert_eq!(resp.status(), StatusCode::NOT_FOUND);
//     }
// }
