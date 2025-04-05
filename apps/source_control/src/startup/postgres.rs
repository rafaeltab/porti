use std::sync::Arc;

use bb8_postgres::{bb8::Pool, PostgresConnectionManager};
use tokio_postgres::NoTls;
use tracing::{error, info, instrument};

use crate::config::PostgresConfig;

#[instrument]
pub async fn setup_postgres(config: &PostgresConfig) -> Arc<Pool<PostgresConnectionManager<NoTls>>> {
    info!("Connecting to postgres");
    let postgres_client = connect_postgres(config).await;

    Arc::new(postgres_client)
}

#[instrument]
async fn connect_postgres(pg_config: &PostgresConfig) -> Pool<PostgresConnectionManager<NoTls>> {
    let mut config = tokio_postgres::Config::default();
    config
        .user(pg_config.user.clone())
        .host(pg_config.host.clone())
        .password(pg_config.password.clone());
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
