use globals::{
    set_global_logger_provider, set_global_meter_provider, set_global_tracer_provider,
    shutdown_logger_provider, shutdown_meter_provider, shutdown_tracer_provider,
};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider as TracerProviderB;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{trace::Sampler, Resource};
use std::time::Duration;
use tracing::subscriber::set_global_default;
use tracing_log::log;
use tracing_opentelemetry::layer;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};

mod globals;

pub fn setup_tracing(
    default_service_name: String,
    env_filter: EnvFilter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Configure the OTLP exporter.  Read from environment variables.
    let otlp_endpoint =
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or("http://otel-collector:4317".to_string());
    let service_name = std::env::var("OTEL_SERVICE_NAME").unwrap_or(default_service_name);

    let otlp_span_exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint.clone())
        .with_timeout(Duration::from_secs(3))
        .build()?;

    let otlp_log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint.clone())
        .with_timeout(Duration::from_secs(3))
        .build()?;

    let otlp_metric_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint.clone())
        .with_timeout(Duration::from_secs(3))
        .build()?;

    // let stdout_span_exporter = opentelemetry_stdout::SpanExporter::default();

    let resource = Resource::builder()
        .with_service_name(service_name.clone())
        .build();

    println!(
        "Starting OTLP with service name {} and endpoint {}",
        service_name, otlp_endpoint
    );

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource.clone())
        // .with_simple_exporter(stdout_exporter)
        // .with_simple_exporter(otlp_exporter)
        .with_batch_exporter(otlp_span_exporter)
        .with_sampler(Sampler::AlwaysOn)
        .build();

    let tracer = tracer_provider.tracer(service_name);
    global::set_tracer_provider(tracer_provider.clone());
    set_global_tracer_provider(tracer_provider);

    let logger_provider = SdkLoggerProvider::builder()
        .with_resource(resource.clone())
        .with_batch_exporter(otlp_log_exporter)
        .build();
    let log_bridge = OpenTelemetryLogBridge::new(&logger_provider);
    set_global_logger_provider(logger_provider);

    log::set_boxed_logger(Box::new(log_bridge))?;
    log::set_max_level(log::LevelFilter::Info);

    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource.clone())
        .with_periodic_exporter(otlp_metric_exporter)
        .build();
    global::set_meter_provider(meter_provider.clone());
    set_global_meter_provider(meter_provider);

    let otel_trace_layer = layer().with_tracer(tracer);

    let subscriber = Registry::default()
        .with(env_filter)
        // .with(fmt_layer)
        .with(otel_trace_layer);

    // Set the global default subscriber.
    set_global_default(subscriber)?;

    Ok(())
}

pub fn shutdown_tracing() {
    shutdown_logger_provider();
    shutdown_tracer_provider();
    shutdown_meter_provider();
    println!("Shutdown");
}
