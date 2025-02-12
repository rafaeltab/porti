use async_trait::async_trait;
use shaku::{Interface, Provider};
use source_control_domain::{
    entities::organization::Organization,
    repositories::organization_repository::{CreateOrganizationError, OrganizationRepository},
};
use thiserror::Error;

pub struct CreateOrganizationCommand {
    pub name: String,
}

#[async_trait]
pub trait CreateOrganizationCommandHandler: Interface {
    async fn handle(
        &self,
        command: CreateOrganizationCommand,
    ) -> Result<Organization, CreateOrganizationCommandError>;
}

#[derive(Provider)]
#[shaku(interface = CreateOrganizationCommandHandler)]
pub struct CreateOrganizationCommandHandlerImpl {
    #[shaku(provide)]
    pub repository: Box<dyn OrganizationRepository>,
}

#[async_trait]
impl CreateOrganizationCommandHandler for CreateOrganizationCommandHandlerImpl {
    async fn handle(
        &self,
        command: CreateOrganizationCommand,
    ) -> Result<Organization, CreateOrganizationCommandError> {
        let res = self.repository.create(command.name).await;

        res.map_err(|err| match err {
            CreateOrganizationError::Connection => CreateOrganizationCommandError::Connection,
            CreateOrganizationError::Unexpected => CreateOrganizationCommandError::Unexpected,
            CreateOrganizationError::Conflict => CreateOrganizationCommandError::Conflict,
        })
    }
}

#[derive(Error, Debug)]
pub enum CreateOrganizationCommandError {
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Another client tried to write to the same aggregate at the same time")]
    Conflict,
}
