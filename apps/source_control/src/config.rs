use std::{env, sync::Arc};

use config::Config;
use serde::{Deserialize, Serialize};

const OTEL_EXPORTER_OTLP_ENDPOINT_ENV_NAME: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
const OTEL_SERVICE_NAME_ENV_NAME: &str = "OTEL_SERVICE_NAME";
const CONFIG_PATH_ENV_NAME: &str = "SOURCE_CONTROL_CONFIG_PATH";

pub fn get_config() -> Arc<SourceControlConfig> {
    let mut config_path = "config.json".to_string();
    if let Ok(path) = env::var(CONFIG_PATH_ENV_NAME) {
        config_path = path;
    }

    let settings = Config::builder()
        .add_source(config::File::with_name(&config_path))
        .add_source(config::Environment::with_prefix("SOURCE_CONTROL"))
        .build()
        .unwrap();

    let mut config: SourceControlConfig =
        settings.try_deserialize().expect("Incorrect configuration");

    if let Ok(exporter_endpoint) = env::var(OTEL_EXPORTER_OTLP_ENDPOINT_ENV_NAME) {
        config.telemetry.open_telemetry_endpoint = exporter_endpoint;
    }

    if let Ok(service_name) = env::var(OTEL_SERVICE_NAME_ENV_NAME) {
        config.telemetry.service_name = service_name;
    }

    Arc::new(config)
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SourceControlConfig {
    pub eventstore: EventStoreConfig,
    pub postgres: PostgresConfig,
    pub telemetry: TelemetryConfig,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct EventStoreConfig {
    pub connection_string: String,
    pub projections: ProjectionsConfig,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionsConfig {
    pub organizations_postgres: ProjectionConfig,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionConfig {
    pub persistent_subscription_name: String,
    pub workers: u32,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostgresConfig {
    pub user: String,
    pub host: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryConfig {
    pub open_telemetry_endpoint: String,
    pub service_name: String,
    pub directives: Vec<String>,
    pub level_directive: String,
}
