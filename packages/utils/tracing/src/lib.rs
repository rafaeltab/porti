use opentelemetry::global;
use opentelemetry::trace::TracerProvider as TracerProviderB;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::{trace::Sampler, Resource};
use std::mem;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;
use tracing::subscriber::set_global_default;
use tracing_log::log;
use tracing_opentelemetry::{layer, OpenTelemetryLayer};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::{layer::SubscriberExt, Registry};

static GLOBAL_LOGGER_PROVIDER: OnceLock<RwLock<SdkLoggerProvider>> = OnceLock::new();
static GLOBAL_TRACER_PROVIDER: OnceLock<RwLock<SdkTracerProvider>> = OnceLock::new();

#[inline]
fn global_tracer_provider() -> &'static RwLock<SdkTracerProvider> {
    GLOBAL_TRACER_PROVIDER.get_or_init(|| RwLock::new(SdkTracerProvider::builder().build()))
}

fn set_global_tracer_provider(new_tracer_provider: SdkTracerProvider) {
    let mut tracer_provider = global_tracer_provider()
        .write()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned");
    let _ = mem::replace(&mut *tracer_provider, new_tracer_provider);
}

pub fn shutdown_tracer_provider() {
    let tracer_provider = global_tracer_provider()
        .read()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned");
    let _ = tracer_provider.shutdown();
}

#[inline]
fn global_logger_provider() -> &'static RwLock<SdkLoggerProvider> {
    GLOBAL_LOGGER_PROVIDER.get_or_init(|| RwLock::new(SdkLoggerProvider::builder().build()))
}

fn set_global_logger_provider(new_logger_provider: SdkLoggerProvider) {
    let mut logger_provider = global_logger_provider()
        .write()
        .expect("GLOBAL_LOGGER_PROVIDER RwLock poisoned");
    let _ = mem::replace(&mut *logger_provider, new_logger_provider);
}

pub fn shutdown_logger_provider() {
    let logger_provider = global_logger_provider()
        .read()
        .expect("GLOBAL_LOGGER_PROVIDER RwLock poisoned");
    let _ = logger_provider.shutdown();
}

pub fn setup_tracing(
    default_service_name: String,
    env_filter: EnvFilter,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Configure the OTLP exporter.  Read from environment variables.
    let otlp_endpoint =
        std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or("http://localhost:4317".to_string());
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

    // let stdout_exporter = opentelemetry_stdout::SpanExporter::default();

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
    log::set_boxed_logger(Box::new(log_bridge)).unwrap();

    let fmt_layer = fmt::layer().with_target(true).compact();

    set_global_logger_provider(logger_provider);
    let otel_trace_layer = layer().with_tracer(tracer);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_trace_layer);

    // Set the global default subscriber.
    set_global_default(subscriber)?;

    Ok(())
}

pub fn shutdown_tracing() {
    shutdown_logger_provider();
    shutdown_tracer_provider();
    println!("Shutdown");
}
