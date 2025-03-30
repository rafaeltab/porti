use std::{
    mem,
    sync::{OnceLock, RwLock},
};

use opentelemetry_sdk::{
    logs::SdkLoggerProvider, metrics::SdkMeterProvider, trace::SdkTracerProvider,
};

static GLOBAL_LOGGER_PROVIDER: OnceLock<RwLock<SdkLoggerProvider>> = OnceLock::new();
static GLOBAL_TRACER_PROVIDER: OnceLock<RwLock<SdkTracerProvider>> = OnceLock::new();
static GLOBAL_METER_PROVIDER: OnceLock<RwLock<SdkMeterProvider>> = OnceLock::new();

#[inline]
fn global_meter_provider() -> &'static RwLock<SdkMeterProvider> {
    GLOBAL_METER_PROVIDER.get_or_init(|| RwLock::new(SdkMeterProvider::builder().build()))
}

pub fn set_global_meter_provider(new_meter_provider: SdkMeterProvider) {
    let mut meter_provider = global_meter_provider()
        .write()
        .expect("GLOBAL_METER_PROVIDER RwLock poisoned");
    let _ = mem::replace(&mut *meter_provider, new_meter_provider);
}

pub fn shutdown_meter_provider() {
    let meter_provider = global_meter_provider()
        .read()
        .expect("GLOBAL_METER_PROVIDER RwLock poisoned");
    let _ = meter_provider.shutdown();
}

#[inline]
fn global_tracer_provider() -> &'static RwLock<SdkTracerProvider> {
    GLOBAL_TRACER_PROVIDER.get_or_init(|| RwLock::new(SdkTracerProvider::builder().build()))
}

pub fn set_global_tracer_provider(new_tracer_provider: SdkTracerProvider) {
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

pub fn set_global_logger_provider(new_logger_provider: SdkLoggerProvider) {
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
