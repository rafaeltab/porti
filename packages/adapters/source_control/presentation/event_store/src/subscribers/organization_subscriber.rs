use std::{sync::Arc, time::SystemTime};

use event_store_util::{
    aggregates::organization::EventStoreOrganizationEvent, from_resolved_event,
};
use eventstore::{
    Client, Error, PersistentSubscription, PersistentSubscriptionToAllOptions, ResolvedEvent,
    SubscribeToPersistentSubscriptionOptions, SubscriptionFilter,
};
use opentelemetry::{
    metrics::{Counter, Histogram},
    KeyValue,
};
use source_control_domain::aggregates::organization::OrganizationEvent;
use source_control_postgres_persistence_adapter::projectors::{Projector, ProjectorError};
use tracing::{error, info, instrument, span, Level, Span};

pub struct OrganizationSubscriber {
    pub client: Arc<Client>,
    pub projector: Box<dyn Projector<OrganizationEvent>>,
    pub subscription_name: String,
    pub worker_id: i32,
    pub metrics: SubscriberMetrics,
}

pub struct SubscriberMetrics {
    pub event_projection_started: Counter<u64>,
    pub event_projection_completed: Counter<u64>,
    pub event_projection_duration_seconds: Histogram<f64>,
}

impl OrganizationSubscriber {
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
        let original_event = event.get_original_event();
        let event_id = original_event.id.to_string();
        let event_type = &original_event.event_type;
        Span::current().record("eventstore.event.id", event_id);
        Span::current().record("eventstore.event.type", event_type);
        let mut attributes = vec![KeyValue::new("eventstore.event.type", event_type.clone())];
        self.metrics.event_projection_started.add(1, &attributes);
        let start = SystemTime::now();
        info!("Begin processing event");

        let organization_event = from_resolved_event::<EventStoreOrganizationEvent>(&event);

        let res = self.projector.project(organization_event.0).await;
        if let Err(err) = res {
            self.handle_error(sub, event, err).await;
            let duration = start.elapsed();
            attributes.push(KeyValue::new("eventstore_event.failure", true));
            self.metrics.event_projection_completed.add(1, &attributes);
            self.metrics.event_projection_duration_seconds.record(
                duration.map(|t| t.as_secs_f64()).unwrap_or_default(),
                &attributes,
            );

            return Ok(());
        };
        sub.ack(event).await?;
        let duration = start.elapsed();
        attributes.push(KeyValue::new("eventstore_event.failure", false));
        self.metrics.event_projection_completed.add(1, &attributes);
        self.metrics.event_projection_duration_seconds.record(
            duration.map(|t| t.as_secs_f64()).unwrap_or_default(),
            &attributes,
        );

        info!("Acknowledged event");
        Ok(())
    }

    #[instrument(skip(sub, self, event), level = "error")]
    async fn handle_error(
        &self,
        sub: &mut PersistentSubscription,
        event: ResolvedEvent,
        err: Box<dyn ProjectorError>,
    ) {
        let span = span!(Level::ERROR, "projection_failure");
        let _span = span.enter();
        error!("Error occurred while running projection {:?}", err);
        let action = match err.get_retryable() {
            true => eventstore::NakAction::Retry,
            false => eventstore::NakAction::Park,
        };
        let res = sub.nack(event, action, "Projection failed, retry").await;
        if let Err(err) = res {
            error!("Error occurred while nacking message {:?}", err);
        }
    }
}
