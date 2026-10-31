use crate::config::TracingConfig;
use opentelemetry::{KeyValue, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::Tracer};
use opentelemetry_semantic_conventions::{
    SCHEMA_URL,
    resource::{SERVICE_NAME, SERVICE_VERSION},
};
use std::time::Duration;
use tracing::level_filters::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn init_otel(endpoint: String) -> Tracer {
    let otel_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otel_exporter)
        .with_resource(
            Resource::builder_empty()
                .with_schema_url(
                    [
                        KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
                        KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                    ],
                    SCHEMA_URL,
                )
                .build(),
        )
        .build();

    opentelemetry::global::set_tracer_provider(tracer_provider.clone());
    tracer_provider.tracer("rustical")
}

pub fn setup_tracing(config: &TracingConfig) {
    let fmt_layer = tracing_subscriber::fmt::layer();
    let filter_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive("h2=warn".parse().unwrap())
        .add_directive("hyper_util=warn".parse().unwrap())
        .add_directive("opentelemetry_sdk=warn".parse().unwrap())
        .add_directive("tower=warn".parse().unwrap());

    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);

    if let Some(endpoint) = &config.endpoint {
        {
            opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
            registry
                .with(OpenTelemetryLayer::new(init_otel(endpoint.to_string())))
                .init();
        }
    } else {
        registry.init();
    }
}
