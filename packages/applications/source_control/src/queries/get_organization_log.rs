use std::sync::Arc;

use source_control_domain::{
    aggregates::organization::OrganizationEvent,
    entities::organization::OrganizationId,
    repositories::organization_repository::{GetOrganizationLogError, OrganizationRepository},
};
use thiserror::Error;

pub struct GetOrganizationLogQuery {
    pub id: u64,
}

pub struct GetOrganizationLogQueryHandler {
    pub repository: Arc<dyn OrganizationRepository>,
}

impl GetOrganizationLogQueryHandler {
    pub async fn handle(
        &self,
        query: GetOrganizationLogQuery,
    ) -> Result<Box<[OrganizationEvent]>, GetOrganizationLogQueryError> {
        match self.repository.get_log(OrganizationId(query.id)).await {
            Ok(organization_log) => Ok(organization_log),
            Err(GetOrganizationLogError::Connection) => {
                Err(GetOrganizationLogQueryError::Connection)
            }
            Err(GetOrganizationLogError::Unexpected) => {
                Err(GetOrganizationLogQueryError::Unexpected)
            }
            Err(GetOrganizationLogError::NotFound { organization_id }) => {
                Err(GetOrganizationLogQueryError::NotFound {
                    organization_id: organization_id.0,
                })
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum GetOrganizationLogQueryError {
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Organization with {organization_id} not found.")]
    NotFound { organization_id: u64 },
}
