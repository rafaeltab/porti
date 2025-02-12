use std::sync::Arc;

use eventstore::Client;
use shaku::{Component, Interface};

pub trait EventStoreProvider: Interface {
    fn get_client(&self) -> Arc<Client>;
}

#[derive(Component)]
#[shaku(interface = EventStoreProvider)]
pub struct EventStoreProviderImpl {
    client: Arc<Client>,
}

impl EventStoreProvider for EventStoreProviderImpl {
    fn get_client(&self) -> Arc<Client> {
        self.client.clone()
    }
}
