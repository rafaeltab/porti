use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;

use async_trait::async_trait;
use event_store_util::aggregates::organization::EventStoreOrganizationEvent;
use event_store_util::from_recorded_event;
use eventstore::{AppendToStreamOptions, Client, EventData, ReadStreamOptions};
use log::error;
use serde_json::json;
use source_control_domain::entities::organization::Organization;
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

pub struct OrganizationRepositoryImpl {
    pub client: Arc<Client>,
}

impl OrganizationRepositoryImpl {
    pub fn new_generic(client: Arc<Client>) -> Arc<dyn OrganizationRepository> {
        Arc::new(Self { client })
    }
}

#[async_trait]
impl OrganizationRepository for OrganizationRepositoryImpl {
    async fn get(
        &self,
        organization_id: OrganizationId,
    ) -> Result<OrganizationAggregate, GetOrganizationError> {
        let stream = OrganizationRepositoryImpl::get_stream_name(organization_id);

        let read_stream_result = self
            .client
            .read_stream(stream.clone(), &ReadStreamOptions::default())
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

    async fn save(&self, organization: OrganizationAggregate) -> Result<(), SaveOrganizationError> {
        let events_opt: Result<Vec<EventData>, SaveOrganizationError> = organization
            .draft_events
            .iter()
            .map(|x| x.to_event_data())
            .map(|x| x.ok_or(SaveOrganizationError::Unexpected))
            .collect();

        let events = events_opt?;

        let stream = OrganizationRepositoryImpl::get_stream_name(organization.root.id);

        let write_result = self
            .client
            .append_to_stream(
                stream,
                &AppendToStreamOptions::default().expected_revision(
                    eventstore::ExpectedRevision::Exact(organization.latest_revision),
                ),
                events,
            )
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

        let write_result = self
            .client
            .append_to_stream(
                stream,
                &AppendToStreamOptions::default()
                    .expected_revision(eventstore::ExpectedRevision::NoStream),
                vec![event_data],
            )
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
