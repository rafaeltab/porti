use serde::Serialize;
use source_control_domain::aggregates::organization::OrganizationEvent;
use utoipa::ToSchema;

use super::platform_account::PlatformAccountDto;

#[derive(Serialize, ToSchema)]
pub enum OrganizationEventDto {
    AddPlatformAccount {
        organization_id: u64,
        account: PlatformAccountDto,
    },
    RemovePlatformAccount {
        organization_id: u64,
        account_id: u64,
    },
    CreateOrganizationEvent {
        organization_id: u64,
        name: String,
    },
}

impl From<&OrganizationEvent> for OrganizationEventDto {
    fn from(value: &OrganizationEvent) -> Self {
        match value {
            OrganizationEvent::AddPlatformAccount {
                organization_id,
                account,
            } => OrganizationEventDto::AddPlatformAccount {
                organization_id: organization_id.0,
                account: account.into(),
            },
            OrganizationEvent::RemovePlatformAccount {
                organization_id,
                account_id,
            } => OrganizationEventDto::RemovePlatformAccount {
                organization_id: organization_id.0,
                account_id: account_id.0,
            },
            OrganizationEvent::CreateOrganizationEvent {
                organization_id,
                name,
            } => OrganizationEventDto::CreateOrganizationEvent {
                organization_id: organization_id.0,
                name: name.clone(),
            },
        }
    }
}
