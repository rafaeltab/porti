use std::sync::Arc;

use tokio_postgres::{tls::NoTlsStream, Client, Connection, NoTls, Socket};
use tracing::{error, info, instrument};

#[instrument]
pub async fn setup_postgres() -> Arc<Client> {
    info!("Connecting to postgres");
    let (postgres_client, postgres_connection) = connect_postgres().await;

    info!("Keeping postgres connection alive");
    tokio::spawn(async move {
        if let Err(e) = postgres_connection.await {
            error!(error = format!("{:?}", e), "postgres connection error");
        }
    });
    Arc::new(postgres_client)
}

#[instrument]
async fn connect_postgres() -> (Client, Connection<Socket, NoTlsStream>) {
    let res = tokio_postgres::Config::default()
        .user("source_control")
        .host("localhost")
        .password("S3cret")
        .connect(NoTls)
        .await;

    match res {
        Ok(client_and_connection) => client_and_connection,
        Err(err) => {
            error!(
                error = format!("{:?}", err),
                "Error while connecting to postgres server"
            );

            panic!();
        }
    }
}
