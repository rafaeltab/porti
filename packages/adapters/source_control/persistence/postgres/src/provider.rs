use std::sync::Arc;

use shaku::{Component, Interface};
use tokio_postgres::Client;

pub trait PostgresProvider: Interface {
    fn get_client(&self) -> Arc<Client>;
}

#[derive(Component)]
#[shaku(interface = PostgresProvider)]
pub struct PostgresProviderImpl {
    client: Arc<Client>,
}

impl PostgresProvider for PostgresProviderImpl {
    fn get_client(&self) -> Arc<Client> {
        self.client.clone()
    }
}
