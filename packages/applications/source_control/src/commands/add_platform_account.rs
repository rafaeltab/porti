use async_trait::async_trait;
use shaku::{Interface, Provider};
use source_control_domain::{
    aggregates::organization::OrganizationError,
    entities::organization::{Organization, OrganizationId},
    factories::platform_account::PlatformAccountFactory,
    repositories::organization_repository::{
        GetOrganizationError, OrganizationRepository, SaveOrganizationError,
    },
};
use thiserror::Error;
use tracing::instrument;

#[derive(Debug)]
pub struct AddPlatformAccountCommand {
    pub organization_id: u64,
    pub name: String,
    pub platform_name: String,
}

#[async_trait]
pub trait AddPlatformAccountCommandHandler: Interface {
    async fn handle(
        &self,
        command: AddPlatformAccountCommand,
    ) -> Result<Organization, AddPlatformAccountCommandError> ;
}

#[derive(Provider)]
#[shaku(interface = AddPlatformAccountCommandHandler)]
pub struct AddPlatformAccountCommandHandlerImpl {
    #[shaku(provide)]
    pub repository: Box<dyn OrganizationRepository>,
    #[shaku(provide)]
    pub platform_account_factory: Box<dyn PlatformAccountFactory>,
}

#[async_trait]
impl AddPlatformAccountCommandHandler for AddPlatformAccountCommandHandlerImpl {
    #[instrument(skip(self))]
    async fn handle(
        &self,
        command: AddPlatformAccountCommand,
    ) -> Result<Organization, AddPlatformAccountCommandError> {
        let mut aggregate = match self
            .repository
            .get(OrganizationId(command.organization_id))
            .await
        {
            Ok(agg) => Ok(agg),
            Err(err) => match err {
                GetOrganizationError::NotFound { organization_id } => {
                    Err(AddPlatformAccountCommandError::NotFound {
                        organization_id: organization_id.0,
                    })
                }
                GetOrganizationError::Connection => Err(AddPlatformAccountCommandError::Connection),
                GetOrganizationError::Unexpected => Err(AddPlatformAccountCommandError::Unexpected),
            },
        }?;

        let platform_account = self.platform_account_factory.create(
            command.name,
            command.platform_name,
            &aggregate.root,
        );

        match aggregate.add_platform_account(platform_account) {
            Ok(_) => Ok(()),
            Err(err) => match err {
                OrganizationError::AccountAlreadyAdded { .. } => {
                    Err(AddPlatformAccountCommandError::AccountAlreadyAdded)
                }
                OrganizationError::AccountNotLinked { .. } => panic!(""),
            },
        }?;
        let root = aggregate.root.clone();

        match self.repository.save(aggregate).await {
            Ok(_) => Ok(root),
            Err(err) => match err {
                SaveOrganizationError::Connection => {
                    Err(AddPlatformAccountCommandError::Connection)
                }
                SaveOrganizationError::Unexpected => {
                    Err(AddPlatformAccountCommandError::Unexpected)
                }
                SaveOrganizationError::Conflict => Err(AddPlatformAccountCommandError::Conflict),
            },
        }
    }
}

#[derive(Error, Debug)]
pub enum AddPlatformAccountCommandError {
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Another client tried to write to the same aggregate at the same time")]
    Conflict,
    #[error("The organization could not be found")]
    NotFound { organization_id: u64 },
    #[error("Account already added")]
    AccountAlreadyAdded,
}
