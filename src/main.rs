use crate::config::Config;
use actix_web::HttpServer;
use anyhow::Result;
use app::make_app;
use clap::Parser;
use config::{DataStoreConfig, SqliteDataStoreConfig, TracingConfig};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{self, BatchConfig, Tracer};
use opentelemetry_sdk::{runtime, Resource};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use rustical_store::auth::StaticUserStore;
use rustical_store::sqlite_store::{create_db_pool, SqliteStore};
use rustical_store::{AddressbookStore, CalendarStore};
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::level_filters::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

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

async fn get_data_stores(
    migrate: bool,
    config: &DataStoreConfig,
) -> Result<(
    Arc<RwLock<dyn AddressbookStore>>,
    Arc<RwLock<dyn CalendarStore>>,
)> {
    Ok(match &config {
        DataStoreConfig::Sqlite(SqliteDataStoreConfig { db_url }) => {
            let db = create_db_pool(db_url, migrate).await?;
            let sqlite_store = Arc::new(RwLock::new(SqliteStore::new(db)));
            (sqlite_store.clone(), sqlite_store.clone())
        }
    })
}

pub fn init_tracer() -> Tracer {
    let otel_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_timeout(Duration::from_secs(1));

    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otel_exporter)
        .with_trace_config(
            trace::Config::default().with_resource(Resource::from_schema_url(
                [
                    KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
                    KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                ],
                SCHEMA_URL,
            )),
        )
        .with_batch_config(BatchConfig::default())
        .install_batch(runtime::Tokio)
        .expect("Failed to install tracer");

    global::set_tracer_provider(tracer_provider.clone());
    tracer_provider.tracer("rustical")
}

fn setup_tracing(config: &TracingConfig) {
    let fmt_layer = tracing_subscriber::fmt::layer();
    let filter_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env_lossy()
        .add_directive("h2=warn".parse().unwrap())
        .add_directive("hyper_util=warn".parse().unwrap())
        .add_directive("tower=warn".parse().unwrap());

    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);

    if config.opentelemetry {
        global::set_text_map_propagator(TraceContextPropagator::new());
        registry.with(OpenTelemetryLayer::new(init_tracer())).init();
    } else {
        registry.init();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config: Config = toml::from_str(&fs::read_to_string(&args.config_file)?)?;

    setup_tracing(&config.tracing);

    let (addr_store, cal_store) = get_data_stores(args.migrate, &config.data_store).await?;

    let user_store = Arc::new(match config.auth {
        config::AuthConfig::Static(config) => StaticUserStore::new(config),
    });

    HttpServer::new(move || make_app(addr_store.clone(), cal_store.clone(), user_store.clone()))
        .bind((config.http.host, config.http.port))?
        .run()
        .await?;

    Ok(())
}
