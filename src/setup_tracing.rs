use crate::config::TracingConfig;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

pub fn init_tracer() -> Tracer {
    let otel_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_timeout(Duration::from_secs(1))
        .build()
        .unwrap();

    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(otel_exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(Resource::from_schema_url(
            [
                KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            ],
            SCHEMA_URL,
        ))
        .build();

    global::set_tracer_provider(tracer_provider.clone());
    tracer_provider.tracer("rustical")
}

pub fn setup_tracing(config: &TracingConfig) {
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
