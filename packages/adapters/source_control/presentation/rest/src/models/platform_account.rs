use serde::Serialize;
use source_control_domain::entities::platform_account::PlatformAccount;
use utoipa::ToSchema;

use super::platform::PlatformDto;

#[derive(Serialize, ToSchema)]
pub struct PlatformAccountDto {
    id: u64,
    name: String,
    platform: PlatformDto,
}

impl From<&PlatformAccount> for PlatformAccountDto {
    fn from(value: &PlatformAccount) -> Self {
        Self {
            id: value.id.0,
            name: value.name.clone(),
            platform: (&value.platform).into(),
        }
    }
}
