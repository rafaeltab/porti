use opentelemetry::trace::TracerProvider as TracerProviderB;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    trace::{Sampler, TracerProvider},
    Resource,
};
use std::time::Duration;
use tracing::subscriber::set_global_default;
use tracing_log::LogTracer;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::{layer::SubscriberExt, Registry};

pub fn setup_tracing(
    default_service_name: String,
    env_filter: EnvFilter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Configure the OTLP exporter.  Read from environment variables.
    let otlp_endpoint =
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or("http://localhost:4317".to_string());
    let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or(default_service_name);

    let otlp_span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint.clone())
        .with_timeout(Duration::from_secs(3))
        .build()?;

    // let stdout_exporter = opentelemetry_stdout::SpanExporter::default();

    let resource = Resource::default().merge(&Resource::new(vec![KeyValue::new(
        "service.name",
        service_name.clone(),
    )]));

    println!(
        "Starting OTLP with service name {} and endpoint {}",
        service_name, otlp_endpoint
    );

    let tracer_provider = TracerProvider::builder()
        .with_resource(resource)
        // .with_simple_exporter(stdout_exporter)
        // .with_simple_exporter(otlp_exporter)
        .with_batch_exporter(otlp_span_exporter, opentelemetry_sdk::runtime::Tokio)
        .with_sampler(Sampler::AlwaysOn)
        .build();

    let tracer = tracer_provider.tracer(service_name);

    global::set_tracer_provider(tracer_provider.clone());

    let fmt_layer = fmt::layer().with_target(true).compact();

    let subscriber = Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .with(OpenTelemetryLayer::new(tracer));

    // Set the global default subscriber.
    set_global_default(subscriber)?;

    LogTracer::init()?;

    Ok(())
}

pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
    println!("Shutdown");
}
