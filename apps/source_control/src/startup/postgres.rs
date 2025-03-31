use std::sync::Arc;

use bb8_postgres::{bb8::Pool, PostgresConnectionManager};
use tokio_postgres::{tls::NoTlsStream, Client, Connection, NoTls, Socket};
use tracing::{error, info, instrument};

#[instrument]
pub async fn setup_postgres() -> Arc<Pool<PostgresConnectionManager<NoTls>>> {
    info!("Connecting to postgres");
    let postgres_client = connect_postgres().await;

    Arc::new(postgres_client)
}

#[instrument]
async fn connect_postgres() -> Pool<PostgresConnectionManager<NoTls>> {
    let mut config = tokio_postgres::Config::default();
    config
        .user("source_control")
        .host("postgres")
        .password("S3cret");
    let manager = PostgresConnectionManager::new(config, NoTls);
    let pool = Pool::builder().max_size(32).build(manager).await;

    match pool {
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
