use std::sync::Arc;

use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use source_control_application::{
    commands::remove_platform_account::RemovePlatformAccountCommandHandler,
    queries::get_organization::GetOrganizationQueryHandler,
};
use source_control_application::{
    commands::{
        add_platform_account::AddPlatformAccountCommandHandler,
        create_organization::CreateOrganizationCommandHandler,
    },
    queries::get_organization_log::GetOrganizationLogQueryHandler,
};
use source_control_domain::factories::platform_account::PlatformAccountFactory;
use source_control_event_store_interface::subscribers::organization_subscriber::OrganizationSubscriber;
use source_control_event_store_persistence_adapter::repositories::organization_repository::OrganizationRepositoryImpl;
use source_control_postgres_persistence_adapter::{
    projectors::organization::OrganizationProjector,
    queries::get_organizations::GetOrganizationsQueryHandler,
};
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

mod startup;

static SUBSCRIPTION_NAME: &str = "organization-projector-002";

#[tokio::main]
#[instrument]
async fn main() -> std::result::Result<(), std::io::Error> {
    let env_filter = EnvFilter::from_default_env()
        .add_directive(LevelFilter::INFO.into())
        .add_directive("tokio_postgres=trace".parse().expect(""))
        .add_directive("eventstore=trace".parse().expect(""))
        .add_directive("tokio_postgres::connection=info".parse().expect(""));

    if let Err(err) = setup_tracing("source_control".to_string(), env_filter) {
        println!("Failed to setup tracing, {:?}", err);
        panic!();
    }

    println!("Starting");
    info!("Starting");

    let eventstore_client_arc = setup_eventstore();
    let postgres_client_arc = setup_postgres().await;

    let projector = OrganizationProjector {
        client: postgres_client_arc.clone(),
    };
    let subscriber = OrganizationSubscriber {
        client: eventstore_client_arc.clone(),
        projector,
        worker_id: 1,
        subscription_name: SUBSCRIPTION_NAME.to_string(),
    };

    let platform_account_factory = PlatformAccountFactory {};
    let platform_account_factory_arc = Arc::new(platform_account_factory);

    let get_organizations_query = GetOrganizationsQueryHandler {
        client: postgres_client_arc.clone(),
    };

    info!("Starting subscriber");
    subscriber
        .prepare_subscription()
        .await
        .expect("Could not prepare subscription");
    tokio::spawn(async move { subscriber.subscribe().await });

    info!("Starting server");
    let val = HttpServer::new(move || {
        let repo = OrganizationRepositoryImpl::new_generic(eventstore_client_arc.clone());

        App::new()
            .wrap(TracingLogger::default())
            .app_data(Data::new(CreateOrganizationCommandHandler {
                repository: repo.clone(),
            }))
            .app_data(Data::new(AddPlatformAccountCommandHandler {
                repository: repo.clone(),
                platform_account_factory: platform_account_factory_arc.clone(),
            }))
            .app_data(Data::new(RemovePlatformAccountCommandHandler {
                repository: repo.clone(),
            }))
            .app_data(Data::new(GetOrganizationQueryHandler {
                repository: repo.clone(),
            }))
            .app_data(Data::new(get_organizations_query.clone()))
            .app_data(Data::new(GetOrganizationLogQueryHandler {
                repository: repo.clone(),
            }))
            .route("/organization", web::post().to(create_organization))
            .route("/organization", web::get().to(get_organizations))
            .route(
                "/organization/{organization_id}",
                web::get().to(get_organization),
            )
            .route(
                "/organization/{organization_id}/log",
                web::get().to(get_organization_log),
            )
            .route(
                "/organization/{organization_id}/platform-account",
                web::post().to(add_platform_account),
            )
            .route(
                "/organization/{organization_id}/platform-account/{platform_account_id}",
                web::delete().to(remove_platform_account),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await;

    shutdown_tracing();

    val
}
