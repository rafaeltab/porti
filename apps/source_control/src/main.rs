use std::sync::Arc;

use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use log::{info, LevelFilter};
use source_control_application::{commands::{
    add_platform_account::AddPlatformAccountCommandHandler,
    create_organization::CreateOrganizationCommandHandler,
}, queries::get_organization_log::GetOrganizationLogQueryHandler};
use source_control_application::queries::get_organization::GetOrganizationQueryHandler;
use source_control_domain::factories::platform_account::PlatformAccountFactory;
use source_control_event_store_interface::subscribers::organization_subscriber::OrganizationSubscriber;
use source_control_event_store_persistence_adapter::repositories::organization_repository::OrganizationRepositoryImpl;
use source_control_postgres_persistence_adapter::projectors::organization::OrganizationProjector;
use source_control_rest_interface::endpoints::organization::{create::create_organization, get_log::get_organization_log, platform_account::add::add_platform_account};
use source_control_rest_interface::endpoints::organization::get::get_organization;
use structured_logger::{async_json::new_writer, Builder};
use tokio_postgres::NoTls;

#[actix_web::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    Builder::with_level(LevelFilter::Debug.as_str())
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    info!("Starting");
    info!("Connecting to event store");
    let eventstore_client = eventstore::Client::new(
        "esdb://localhost:2113?tls=false"
            .parse()
            .expect("Could not connect to event store"),
    )
    .expect("Failed to connect to event store");
    let eventstore_client_arc = Arc::new(eventstore_client);
    info!("Connecting to postgres");
    let (postgres_client, postgres_connection) = tokio_postgres::Config::default()
        .user("source_control")
        .host("localhost")
        .password("S3cret")
        .connect(NoTls)
        .await
        .expect("Failed to connect to postgres");
    let postgres_client_arc = Arc::new(postgres_client);

    info!("Keeping postgres connection alive");
    tokio::spawn(async move {
        if let Err(e) = postgres_connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let projector = OrganizationProjector {
        client: postgres_client_arc,
    };
    let subscriber = OrganizationSubscriber {
        client: eventstore_client_arc.clone(),
        projector,
    };

    let platform_account_factory = PlatformAccountFactory {};
    let platform_account_factory_arc = Arc::new(platform_account_factory);

    info!("Starting subscriber");
    subscriber
        .prepare_subscription()
        .await
        .expect("Could not prepare subscription");
    tokio::spawn(async move { subscriber.subscribe().await });

    info!("Starting server");
    HttpServer::new(move || {
        let repo = OrganizationRepositoryImpl::new_generic(eventstore_client_arc.clone());

        App::new()
            .app_data(Data::new(CreateOrganizationCommandHandler {
                repository: repo.clone(),
            }))
            .app_data(Data::new(AddPlatformAccountCommandHandler {
                repository: repo.clone(),
                platform_account_factory: platform_account_factory_arc.clone(),
            }))
            .app_data(Data::new(GetOrganizationQueryHandler {
                repository: repo.clone(),
            }))
            .app_data(Data::new(GetOrganizationLogQueryHandler {
                repository: repo.clone(),
            }))
            .route("/organization", web::post().to(create_organization))
            .route("/organization/{organization_id}", web::get().to(get_organization))
            .route("/organization/{organization_id}/log", web::get().to(get_organization_log))
            .route("/organization/{organization_id}/platform-account", web::post().to(add_platform_account))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
