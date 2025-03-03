use serde::Serialize;
use source_control_domain::entities::platform::Platform;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct PlatformDto {
    name: String,
}

impl From<&Platform> for PlatformDto {
    fn from(value: &Platform) -> Self {
        Self {
            name: value.name.clone(),
        }
    }
}
