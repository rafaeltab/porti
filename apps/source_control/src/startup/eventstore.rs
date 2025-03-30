use std::sync::Arc;

use eventstore::Client;
use tracing::{error, info, instrument};

#[instrument]
pub fn setup_eventstore() -> Arc<Client> {
    info!("Connecting to event store");
    let connect_result = eventstore::Client::new(
        "esdb://eventstore.db:2113?tls=false"
            .parse()
            .expect("Could not connect to event store"),
    );
    let eventstore_client = match connect_result {
        Ok(client) => client,
        Err(err) => {
            error!(
                error = format!("{:?}", err),
                "Error while connecting to event store"
            );
            panic!();
        }
    };

    Arc::new(eventstore_client)
}
