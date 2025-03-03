use serde::Serialize;
use source_control_domain::entities::organization::Organization;
use source_control_postgres_persistence_adapter::queries::get_organizations::OrganizationResult;
use utoipa::ToSchema;

use super::platform_account::PlatformAccountDto;

#[derive(Serialize, ToSchema)]
pub struct OrganizationDto {
    id: u64,
    name: String,
    platform_accounts: Vec<PlatformAccountDto>,
}

impl From<&Organization> for OrganizationDto {
    fn from(value: &Organization) -> Self {
        Self {
            id: value.id.0,
            name: value.name.clone(),
            platform_accounts: value.platform_accounts.iter().map(|x| x.into()).collect(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct PartialOrganizationDto {
    id: u64,
    name: String,
    platform_account_count: i64,
}

impl From<&OrganizationResult> for PartialOrganizationDto {
    fn from(value: &OrganizationResult) -> Self {
        Self {
            id: value.id.0,
            name: value.name.clone(),
            platform_account_count: value.paltform_account_count,
        }
    }
}
