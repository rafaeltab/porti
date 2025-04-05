use std::sync::Arc;

use actix_tracing_util::MeterFactory;
use actix_web::{web::Data, App, HttpServer};
use config::get_config;
use metrics::{request_metrics, subscriber_metrics};
use myopenapi::WithOpenApi;
use shaku::HasProvider;
use source_control_application::module::{get_module, ApplicationModule};
use source_control_domain::aggregates::organization::OrganizationEvent;
use source_control_event_store_interface::subscribers::organization_subscriber::OrganizationSubscriber;
use source_control_postgres_persistence_adapter::projectors::Projector;
use source_control_rest_interface::endpoints::organization::{
    create::create_organization, get_log::get_organization_log,
    platform_account::add::add_platform_account,
};
use source_control_rest_interface::endpoints::organization::{
    get::get_organization, get_all::get_organizations,
    platform_account::remove::remove_platform_account,
};
use startup::{eventstore::setup_eventstore, postgres::setup_postgres};
use tracing::{info, instrument};
use tracing_actix_web::TracingLogger;
use tracing_core::LevelFilter;
use tracing_subscriber::EnvFilter;
use tracing_util::{setup_tracing, shutdown_tracing};
use utoipa_actix_web::AppExt;

mod config;
mod metrics;
mod startup;

#[tokio::main]
#[instrument]
async fn main() -> std::result::Result<(), std::io::Error> {
    let config = get_config();

    let mut env_filter = EnvFilter::from_default_env();

    let level_filter = match config.telemetry.level_directive.to_lowercase().as_str() {
        "off" => LevelFilter::OFF,
        "error" => LevelFilter::ERROR,
        "warn" => LevelFilter::WARN,
        "info" => LevelFilter::INFO,
        "debug" => LevelFilter::DEBUG,
        "trace" => LevelFilter::TRACE,
        _ => panic!("Incorrect level filter passed for configuration"),
    };

    env_filter = env_filter.add_directive(level_filter.into());

    for directive in &config.telemetry.directives {
        env_filter = env_filter.add_directive(
            directive
                .parse()
                .expect("Incorrect directive passed for configuration"),
        );
    }

    if let Err(err) = setup_tracing(
        config.telemetry.service_name.clone(),
        config.telemetry.open_telemetry_endpoint.clone(),
        env_filter,
    ) {
        panic!("Failed to setup tracing, {:?}", err);
    }

    println!("Starting");
    info!("Starting");

    let eventstore_client_arc = setup_eventstore(&config.eventstore.connection_string);
    let postgres_client_arc = setup_postgres(&config.postgres).await;

    let module = Arc::new(get_module(
        postgres_client_arc.clone(),
        eventstore_client_arc.clone(),
    ));

    for i in 0..config.eventstore.projections.organizations_postgres.workers {
        let projector: Box<dyn Projector<OrganizationEvent>> = module.provide().unwrap();
        let subscriber = OrganizationSubscriber {
            client: eventstore_client_arc.clone(),
            projector,
            worker_id: i,
            subscription_name: config
                .eventstore
                .projections
                .organizations_postgres
                .persistent_subscription_name
                .clone(),
            metrics: subscriber_metrics(),
        };

        info!("Starting subscriber");
        subscriber
            .prepare_subscription()
            .await
            .expect("Could not prepare subscription");
        tokio::spawn(async move { subscriber.subscribe().await });
    }
    let metrics = request_metrics();

    info!("Starting server");
    let val = HttpServer::new(move || {
        let app_data: Data<ApplicationModule> = module.clone().into();
        App::new()
            .wrap(TracingLogger::default())
            .wrap(MeterFactory {
                metrics: metrics.clone(),
            })
            .into_utoipa_app()
            .app_data(app_data)
            .service(get_organization)
            .service(create_organization)
            .service(get_organizations)
            .service(get_organization_log)
            .service(add_platform_account)
            .service(remove_platform_account)
            .with_openapi()
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    shutdown_tracing();

    val
}

mod myopenapi {
    use std::sync::RwLock;

    use actix_web::{
        dev::{ServiceFactory, ServiceRequest},
        get, App, HttpResponse, Responder,
    };
    use utoipa_actix_web::UtoipaApp;

    pub trait WithOpenApi<T> {
        fn with_openapi(self) -> App<T>;
    }
    static OPENAPI_STR: RwLock<String> = RwLock::new(String::new());

    #[get("openapi.json", name = "openapi")]
    async fn get_openapi() -> impl Responder {
        let val = OPENAPI_STR.read().unwrap().clone();
        HttpResponse::Ok()
            .content_type("application/vnd.oai.openapi+json;version=3.1")
            .body(val)
    }

    impl<T> WithOpenApi<T> for UtoipaApp<T>
    where
        T: ServiceFactory<ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>,
    {
        fn with_openapi(self) -> App<T> {
            let (a, b) = self.split_for_parts();
            {
                let mut w = OPENAPI_STR.write().unwrap();
                *w = b.to_json().unwrap();
            }

            a.service(get_openapi)
        }
    }
}
