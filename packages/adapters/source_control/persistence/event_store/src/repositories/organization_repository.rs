use source_control_domain::entities::organization::Organization;
use async_trait::async_trait;
use eventstore::{AppendToStreamOptions, Client, EventData, ReadStreamOptions};
use log::error;
use serde_json::json;
use source_control_domain::{
    aggregates::{
        base::DomainEvent,
        organization::{OrganizationAggregate, OrganizationEvents},
    },
    entities::{
        organization::OrganizationId,
        platform::Platform,
        platform_account::{PlatformAccount, PlatformAccountId},
    },
    repositories::organization_repository::{
        CreateOrganizationError, GetOrganizationError, OrganizationRepository,
        SaveOrganizationError,
    },
};

pub struct OrganizationRepositoryImpl {
    client: Client,
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
                    let data = &original_event.data;
                    latest_revision = original_event.revision;
                    match serde_json::from_slice::<serde_json::Value>(data) {
                        Ok(parsed) => events.push((parsed, original_event.event_type.clone())),
                        Err(err) => {
                            error!(
                                "Error while deserializing event from stream {}: {}",
                                stream, err
                            );
                            return Err(GetOrganizationError::Unexpected);
                        }
                    }
                }

                let typed_event_opts: Result<Vec<OrganizationEvents>, GetOrganizationError> =
                    events
                        .iter()
                        .map(|event| OrganizationEvents::from_json(&event.0, &event.1))
                        .map(|opt| opt.ok_or(GetOrganizationError::Unexpected))
                        .collect();

                let typed_events = typed_event_opts?;

                Ok(OrganizationAggregate::from_events(
                    typed_events,
                    latest_revision,
                ))
            }
            _ => Err(GetOrganizationError::Connection),
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
        let id = OrganizationId(0);
        let stream = OrganizationRepositoryImpl::get_stream_name(id);

        let event = OrganizationEvents::CreateOrganizationEvent {
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
    fn from_json(value: &serde_json::Value, event_type: &str) -> Option<Self>;
}

impl DomainEventJson for OrganizationEvents {
    fn to_event_data(&self) -> Option<EventData> {
        let json = match self {
            OrganizationEvents::AddPlatformAccount {
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
            OrganizationEvents::RemovePlatformAccount {
                account_id,
                organization_id,
            } => json!({
                "organization_id": organization_id.0,
                "account": {
                    "id": account_id.0
                }
            }),
            OrganizationEvents::CreateOrganizationEvent {
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

    fn from_json(value: &serde_json::Value, event_type: &str) -> Option<Self> {
        match event_type {
            "Porti.SourceControl/Aggregates/Organization/AddPlatformAccount/1" => {
                let organization_id = &value["organization_id"].as_u64()?;
                let account_id = &value["account"]["id"].as_u64()?;
                let account_name = &value["account"]["name"].as_str()?;
                let platform_name = &value["account"]["platform"]["name"].as_str()?;

                Some(OrganizationEvents::AddPlatformAccount {
                    organization_id: OrganizationId(*organization_id),
                    account: PlatformAccount {
                        id: PlatformAccountId(*account_id),
                        name: account_name.to_string(),
                        platform: Platform {
                            name: platform_name.to_string(),
                        },
                    },
                })
            }
            "Porti.SourceControl/Aggregates/Organization/RemovePlatformAccount/1" => {
                let organization_id = &value["organization_id"].as_u64()?;
                let account_id = &value["account"]["id"].as_u64()?;

                Some(OrganizationEvents::RemovePlatformAccount {
                    account_id: PlatformAccountId(*account_id),
                    organization_id: OrganizationId(*organization_id),
                })
            }
            "Porti.SourceControl/Aggregates/Organization/Create/1" => {
                let organization_id = &value["organization_id"].as_u64()?;
                let name = &value["name"].as_str()?;

                Some(OrganizationEvents::CreateOrganizationEvent {
                    organization_id: OrganizationId(*organization_id),
                    name: name.to_string(),
                })
            }
            _ => None,
        }
    }
}
