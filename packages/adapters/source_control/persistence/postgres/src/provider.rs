use std::sync::Arc;

use async_trait::async_trait;
use bb8_postgres::{
    bb8::{Pool, PooledConnection},
    PostgresConnectionManager,
};
use shaku::{Component, Interface};
use tokio_postgres::NoTls;

#[async_trait]
pub trait PostgresProvider: Interface {
    async fn get_client(&self) -> PooledConnection<'_, PostgresConnectionManager<NoTls>>;
}

#[derive(Component)]
#[shaku(interface = PostgresProvider)]
pub struct PostgresProviderImpl {
    client: Arc<Pool<PostgresConnectionManager<NoTls>>>,
}

#[async_trait]
impl PostgresProvider for PostgresProviderImpl {
    async fn get_client(&self) -> PooledConnection<'_, PostgresConnectionManager<NoTls>> {
        self.client
            .get()
            .await
            .expect("Something happened while getting a connection")
    }
}
