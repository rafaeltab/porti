use async_trait::async_trait;
use shaku::{Interface, Provider};
use source_control_domain::{
    aggregates::organization::OrganizationEvent,
    entities::organization::OrganizationId,
    repositories::organization_repository::{GetOrganizationLogError, OrganizationRepository},
};
use thiserror::Error;

pub struct GetOrganizationLogQuery {
    pub id: u64,
}

#[async_trait]
pub trait GetOrganizationLogQueryHandler: Interface {
    async fn handle(
        &self,
        query: GetOrganizationLogQuery,
    ) -> Result<Box<[OrganizationEvent]>, GetOrganizationLogQueryError>;
}

#[derive(Provider)]
#[shaku(interface = GetOrganizationLogQueryHandler)]
pub struct GetOrganizationLogQueryHandlerImpl {
    #[shaku(provide)]
    pub repository: Box<dyn OrganizationRepository>,
}

#[async_trait]
impl GetOrganizationLogQueryHandler for GetOrganizationLogQueryHandlerImpl {
    async fn handle(
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
