use std::sync::Arc;

use event_store_util::{
    aggregates::organization::EventStoreOrganizationEvent, from_resolved_event,
};
use eventstore::{
    Client, Error, PersistentSubscription, PersistentSubscriptionToAllOptions, ResolvedEvent,
    SubscribeToPersistentSubscriptionOptions, SubscriptionFilter,
};
use source_control_domain::aggregates::organization::OrganizationEvent;
use source_control_postgres_persistence_adapter::projectors::Projector;
use tracing::{error, info, instrument, span, Level, Span};

pub struct OrganizationSubscriber<TProjector: Projector<OrganizationEvent>> {
    pub client: Arc<Client>,
    pub projector: TProjector,
    pub subscription_name: String,
    pub worker_id: i32,
}

impl<TProjector: Projector<OrganizationEvent>> OrganizationSubscriber<TProjector> {
    pub async fn prepare_subscription(&self) -> Result<(), Error> {
        let subscription = self
            .client
            .create_persistent_subscription_to_all(
                self.subscription_name.clone(),
                &PersistentSubscriptionToAllOptions::default()
                    .start_from(eventstore::StreamPosition::Start)
                    .filter(
                        SubscriptionFilter::on_stream_name()
                            .add_prefix("Porti.SourceControl/Aggregates/Organization/"),
                    )
                    .consumer_strategy_name(eventstore::SystemConsumerStrategy::Pinned),
            )
            .await;

        match subscription {
            Ok(_) => Ok(()),
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
                self.subscription_name.clone(),
                &SubscribeToPersistentSubscriptionOptions::default(),
            )
            .await?;

        loop {
            let event = sub.next().await?;
            self.handle_next(&mut sub, event).await?;
        }
    }

    #[instrument(skip(sub, self))]
    async fn handle_next(
        &self,
        sub: &mut PersistentSubscription,
        event: ResolvedEvent,
    ) -> eventstore::Result<()> {
        let event_id = event.get_original_event().id.to_string();
        Span::current().record("event_id", event_id);
        info!("Begin processing event");

        let organization_event = from_resolved_event::<EventStoreOrganizationEvent>(&event);

        let res = self.projector.project(organization_event.0).await;
        if let Err(err) = res {
            self.handle_error(sub, event, err).await;
            return Ok(());
        };
        sub.ack(event).await?;
        info!("Acknowledged event");
        Ok(())
    }

    #[instrument(skip(sub, self, event), level = "error")]
    async fn handle_error(
        &self,
        sub: &mut PersistentSubscription,
        event: ResolvedEvent,
        err: TProjector::Error,
    ) {
        let span = span!(Level::ERROR, "projection_failure");
        let _span = span.enter();
        error!("Error occurred while running projection {:?}", err);
        let res = sub
            .nack(
                event,
                eventstore::NakAction::Retry,
                "Projection failed, retry",
            )
            .await;
        if let Err(err) = res {
            error!("Error occurred while nacking message {:?}", err);
        }
    }
}
