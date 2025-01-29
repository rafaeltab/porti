use std::sync::Arc;

use event_store_util::{
    aggregates::organization::EventStoreOrganizationEvent, from_resolved_event,
};
use eventstore::{
    Client, Error, PersistentSubscriptionToAllOptions, SubscribeToPersistentSubscriptionOptions,
    SubscriptionFilter,
};
use log::error;
use source_control_domain::aggregates::organization::OrganizationEvent;
use source_control_postgres_persistence_adapter::projectors::Projector;

pub struct OrganizationSubscriber<TProjector: Projector<OrganizationEvent>> {
    pub client: Arc<Client>,
    pub projector: TProjector,
}

impl<TProjector: Projector<OrganizationEvent>> OrganizationSubscriber<TProjector> {
    fn subscription_name() -> &'static str {
        "organization-postgres-projector"
    }

    pub async fn prepare_subscription(&self) -> Result<(), Error> {
        let subscription = self
            .client
            .create_persistent_subscription_to_all(
                Self::subscription_name(),
                &PersistentSubscriptionToAllOptions::default()
                    .filter(
                        SubscriptionFilter::on_stream_name()
                            .add_prefix("Porti.SourceControl/Aggregates/Organization/"),
                    )
                    .consumer_strategy_name(eventstore::SystemConsumerStrategy::Pinned),
            )
            .await;

        match subscription {
            Ok(_) => todo!(),
            Err(err) => match err {
                eventstore::Error::ResourceAlreadyExists => Ok(()),
                _ => Err(err),
            },
        }
    }

    pub async fn subscribe(&self) -> eventstore::Result<()> {
        let mut sub = self
            .client
            .subscribe_to_persistent_subscription_to_all(
                Self::subscription_name(),
                &SubscribeToPersistentSubscriptionOptions::default(),
            )
            .await?;

        loop {
            let event = sub.next().await?;

            let organization_event = from_resolved_event::<EventStoreOrganizationEvent>(&event);
            if (self.projector.project(organization_event.0).await).is_err() {
                error!("Error occurred while running projection")
            };
            sub.ack(event).await?;
        }
    }
}
