use std::sync::Arc;

use source_control_domain::{
    entities::organization::{Organization, OrganizationId},
    repositories::organization_repository::{GetOrganizationError, OrganizationRepository},
};
use thiserror::Error;

pub struct GetOrganizationQuery {
    pub id: u64,
}

#[derive(Debug)]
pub struct GetOrganizationQueryHandler {
    pub repository: Arc<dyn OrganizationRepository>,
}

impl GetOrganizationQueryHandler {
    pub async fn handle(
        &self,
        query: GetOrganizationQuery,
    ) -> Result<Organization, GetOrganizationQueryError> {
        match self.repository.get(OrganizationId(query.id)).await {
            Ok(organization_aggregate) => Ok(organization_aggregate.root),
            Err(GetOrganizationError::Connection) => Err(GetOrganizationQueryError::Connection),
            Err(GetOrganizationError::Unexpected) => Err(GetOrganizationQueryError::Unexpected),
            Err(GetOrganizationError::NotFound { organization_id }) => {
                Err(GetOrganizationQueryError::NotFound {
                    organization_id: organization_id.0,
                })
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum GetOrganizationQueryError {
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Organization with {organization_id} not found.")]
    NotFound { organization_id: u64 },
}
