use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use async_trait::async_trait;
use event_store_util::aggregates::organization::EventStoreOrganizationEvent;
use event_store_util::from_recorded_event;
use eventstore::{AppendToStreamOptions, EventData, ReadStreamOptions};
use serde_json::json;
use shaku::Provider;
use source_control_domain::entities::organization::Organization;
use source_control_domain::repositories::organization_repository::GetOrganizationLogError;
use source_control_domain::{
    aggregates::{
        base::DomainEvent,
        organization::{OrganizationAggregate, OrganizationEvent},
    },
    entities::organization::OrganizationId,
    repositories::organization_repository::{
        CreateOrganizationError, GetOrganizationError, OrganizationRepository,
        SaveOrganizationError,
    },
};
use tracing::{error, instrument, span, Instrument, Level};

use crate::provider::EventStoreProvider;

#[derive(Provider)]
#[shaku(interface = OrganizationRepository)]
pub struct OrganizationRepositoryImpl {
    #[shaku(inject)]
    pub client: Arc<dyn EventStoreProvider>,
}

#[async_trait]
impl OrganizationRepository for OrganizationRepositoryImpl {
    #[instrument(skip(self))]
    async fn get_log(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Box<[OrganizationEvent]>, GetOrganizationLogError> {
        let stream = OrganizationRepositoryImpl::get_stream_name(organization_id);

        let read_span = span!(Level::INFO, "event_store_read_stream");
        let read_stream_result = self
            .client
            .get_client()
            .read_stream(stream.clone(), &ReadStreamOptions::default())
            .instrument(read_span)
            .await;

        match read_stream_result {
            Ok(mut event_stream) => {
                let mut events = Vec::new();
                while let Ok(Some(event)) = event_stream.next().await {
                    let original_event = event.get_original_event();
                    let ev = from_recorded_event::<EventStoreOrganizationEvent>(original_event);
                    events.push(ev.0);
                }

                if events.is_empty() {
                    return Err(GetOrganizationLogError::NotFound { organization_id });
                }

                Ok(events.into())
            }
            Err(err) => match err {
                eventstore::Error::ConnectionClosed => Err(GetOrganizationLogError::Connection),
                eventstore::Error::Grpc { .. } => Err(GetOrganizationLogError::Connection),
                eventstore::Error::GrpcConnectionError(..) => {
                    Err(GetOrganizationLogError::Connection)
                }
                eventstore::Error::AccessDenied => Err(GetOrganizationLogError::Connection),
                eventstore::Error::DeadlineExceeded => Err(GetOrganizationLogError::Connection),
                eventstore::Error::ResourceNotFound => {
                    Err(GetOrganizationLogError::NotFound { organization_id })
                }
                _ => Err(GetOrganizationLogError::Unexpected),
            },
        }
    }

    #[instrument(skip(self))]
    async fn get(
        &self,
        organization_id: OrganizationId,
    ) -> Result<OrganizationAggregate, GetOrganizationError> {
        let stream = OrganizationRepositoryImpl::get_stream_name(organization_id);

        let read_span = span!(Level::INFO, "event_store_read_stream");
        let read_stream_result = self
            .client
            .get_client()
            .read_stream(stream.clone(), &ReadStreamOptions::default())
            .instrument(read_span)
            .await;

        let mut latest_revision: u64 = 0;

        match read_stream_result {
            Ok(mut event_stream) => {
                let mut events = Vec::new();
                while let Ok(Some(event)) = event_stream.next().await {
                    let original_event = event.get_original_event();
                    latest_revision = original_event.revision;
                    let ev = from_recorded_event::<EventStoreOrganizationEvent>(original_event);
                    events.push(ev.0);
                }

                if events.is_empty() {
                    return Err(GetOrganizationError::NotFound { organization_id });
                }

                Ok(OrganizationAggregate::from_events(events, latest_revision))
            }
            Err(err) => match err {
                eventstore::Error::ConnectionClosed => Err(GetOrganizationError::Connection),
                eventstore::Error::Grpc { .. } => Err(GetOrganizationError::Connection),
                eventstore::Error::GrpcConnectionError(..) => Err(GetOrganizationError::Connection),
                eventstore::Error::AccessDenied => Err(GetOrganizationError::Connection),
                eventstore::Error::DeadlineExceeded => Err(GetOrganizationError::Connection),
                eventstore::Error::ResourceNotFound => {
                    Err(GetOrganizationError::NotFound { organization_id })
                }
                _ => Err(GetOrganizationError::Unexpected),
            },
        }
    }

    #[instrument(skip(self, organization))]
    async fn save(&self, organization: OrganizationAggregate) -> Result<(), SaveOrganizationError> {
        let events_opt: Result<Vec<EventData>, SaveOrganizationError> = organization
            .draft_events
            .iter()
            .map(|x| x.to_event_data())
            .map(|x| x.ok_or(SaveOrganizationError::Unexpected))
            .collect();

        let events = events_opt?;

        let stream = OrganizationRepositoryImpl::get_stream_name(organization.root.id);

        let write_span = span!(Level::INFO, "event_store_append_stream");
        let write_result = self
            .client
            .get_client()
            .append_to_stream(
                stream,
                &AppendToStreamOptions::default().expected_revision(
                    eventstore::ExpectedRevision::Exact(organization.latest_revision),
                ),
                events,
            )
            .instrument(write_span)
            .await;

        match write_result {
            Ok(_) => Ok(()),
            Err(err) => match err {
                eventstore::Error::WrongExpectedVersion { .. } => {
                    Err(SaveOrganizationError::Conflict)
                }
                eventstore::Error::ConnectionClosed => Err(SaveOrganizationError::Connection),
                _ => {
                    error!("Error occurred while saving organization: {}", err);
                    Err(SaveOrganizationError::Unexpected)
                }
            },
        }
    }

    #[instrument(skip(self))]
    async fn create(&self, name: String) -> Result<Organization, CreateOrganizationError> {
        let mut hasher = DefaultHasher::default();
        name.hash(&mut hasher);

        let id = OrganizationId(hasher.finish());
        let stream = OrganizationRepositoryImpl::get_stream_name(id);

        let event = OrganizationEvent::CreateOrganizationEvent {
            organization_id: id,
            name: name.clone(),
        };

        let event_data = event
            .to_event_data()
            .ok_or(CreateOrganizationError::Unexpected)?;

        let write_span = span!(Level::INFO, "event_store_append_stream");
        let write_result = self
            .client
            .get_client()
            .append_to_stream(
                stream,
                &AppendToStreamOptions::default()
                    .expected_revision(eventstore::ExpectedRevision::NoStream),
                vec![event_data],
            )
            .instrument(write_span)
            .await;

        match write_result {
            Ok(_) => Ok(Organization {
                id,
                name,
                platform_accounts: vec![],
            }),
            Err(err) => match err {
                eventstore::Error::WrongExpectedVersion { .. } => {
                    Err(CreateOrganizationError::Conflict)
                }
                eventstore::Error::ConnectionClosed => Err(CreateOrganizationError::Connection),
                _ => {
                    error!("Error occurred while saving organization: {}", err);
                    Err(CreateOrganizationError::Unexpected)
                }
            },
        }
    }
}

impl OrganizationRepositoryImpl {
    fn get_stream_name(organization_id: OrganizationId) -> String {
        format!(
            "Porti.SourceControl/Aggregates/Organization/{}",
            organization_id
        )
    }
}

trait DomainEventJson
where
    Self: Sized,
{
    fn to_event_data(&self) -> Option<EventData>;
}

impl DomainEventJson for OrganizationEvent {
    fn to_event_data(&self) -> Option<EventData> {
        let json = match self {
            OrganizationEvent::AddPlatformAccount {
                account,
                organization_id,
            } => json!({
                "organization_id": organization_id.0,
                "account": {
                    "id": account.id.0,
                    "name": account.name,
                    "platform": {
                        "name": account.platform.name
                    }
                }
            }),
            OrganizationEvent::RemovePlatformAccount {
                account_id,
                organization_id,
            } => json!({
                "organization_id": organization_id.0,
                "account": {
                    "id": account_id.0
                }
            }),
            OrganizationEvent::CreateOrganizationEvent {
                organization_id,
                name,
            } => json!({
                "organization_id": organization_id.0,
                "name": name.clone()
            }),
        };

        match EventData::json(format!("{}/1", self.get_event_type()), json) {
            Ok(data) => Some(data),
            Err(err) => {
                error!("{}", err);
                None
            }
        }
    }
}
