use std::{io, sync::Arc};

use actix_web::{
    web::{self, Data},
    App, HttpServer,
};
use log::{info, LevelFilter};
use source_control_application::commands::create_organization::CreateOrganizationCommandHandler;
use source_control_application::queries::get_organization::GetOrganizationQueryHandler;
use source_control_event_store_interface::subscribers::organization_subscriber::OrganizationSubscriber;
use source_control_event_store_persistence_adapter::repositories::organization_repository::OrganizationRepositoryImpl;
use source_control_postgres_persistence_adapter::projectors::organization::OrganizationProjector;
use source_control_rest_interface::endpoints::organization::create::create_organization;
use source_control_rest_interface::endpoints::organization::get::get_organization;
use structured_logger::{async_json::new_writer, Builder};
use tokio_postgres::NoTls;

#[actix_web::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    Builder::with_level(LevelFilter::Debug.as_str())
        .with_target_writer("*", new_writer(tokio::io::stdout()))
        .init();

    let eventstore_client =
        eventstore::Client::new("esdb://localhost:2113?tls=false".parse().expect(""))
            .expect("Failed to connect to event store");
    let eventstore_client_arc = Arc::new(eventstore_client);
    let (postgres_client, postgres_connection) =
        tokio_postgres::connect("host=localhost user=source_control", NoTls)
            .await
            .expect("Failed to connect to postgres");
    let postgres_client_arc = Arc::new(postgres_client);

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

    subscriber.prepare_subscription().await.expect("");
    tokio::spawn(async move {
        subscriber.subscribe().await
    });

    info!("Starting server");
    HttpServer::new(move || {
        let repo = OrganizationRepositoryImpl::new_generic(eventstore_client_arc.clone());

        App::new()
            .app_data(Data::new(CreateOrganizationCommandHandler {
                repository: repo.clone(),
            }))
            .app_data(Data::new(GetOrganizationQueryHandler {
                repository: repo.clone(),
            }))
            .route("/organization", web::post().to(create_organization))
            .route("/organization/{id}", web::get().to(get_organization))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
