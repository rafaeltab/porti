use std::sync::Arc;

use source_control_domain::{
    aggregates::organization::OrganizationError,
    entities::{
        organization::{Organization, OrganizationId},
        platform_account::PlatformAccountId,
    },
    repositories::organization_repository::{
        GetOrganizationError, OrganizationRepository, SaveOrganizationError,
    },
};
use thiserror::Error;

pub struct RemovePlatformAccountCommand {
    pub organization_id: u64,
    pub account_id: u64,
}

#[derive(Debug)]
pub struct RemovePlatformAccountCommandHandler {
    pub repository: Arc<dyn OrganizationRepository>,
}

impl RemovePlatformAccountCommandHandler {
    pub async fn handle(
        &self,
        command: RemovePlatformAccountCommand,
    ) -> Result<Organization, RemovePlatformAccountCommandError> {
        let mut aggregate = match self
            .repository
            .get(OrganizationId(command.organization_id))
            .await
        {
            Ok(agg) => Ok(agg),
            Err(err) => match err {
                GetOrganizationError::NotFound { organization_id } => {
                    Err(RemovePlatformAccountCommandError::OrganizationNotFound {
                        organization_id: organization_id.0,
                    })
                }
                GetOrganizationError::Connection => {
                    Err(RemovePlatformAccountCommandError::Connection)
                }
                GetOrganizationError::Unexpected => {
                    Err(RemovePlatformAccountCommandError::Unexpected)
                }
            },
        }?;

        match aggregate.remove_platform_account(PlatformAccountId(command.account_id)) {
            Ok(_) => Ok(()),
            Err(err) => match err {
                OrganizationError::AccountAlreadyAdded { .. } => panic!(""),
                OrganizationError::AccountNotLinked { .. } => {
                    Err(RemovePlatformAccountCommandError::AccountNotFound {
                        account_id: command.account_id,
                    })
                }
            },
        }?;
        let root = aggregate.root.clone();

        match self.repository.save(aggregate).await {
            Ok(_) => Ok(root),
            Err(err) => match err {
                SaveOrganizationError::Connection => {
                    Err(RemovePlatformAccountCommandError::Connection)
                }
                SaveOrganizationError::Unexpected => {
                    Err(RemovePlatformAccountCommandError::Unexpected)
                }
                SaveOrganizationError::Conflict => Err(RemovePlatformAccountCommandError::Conflict),
            },
        }
    }
}

#[derive(Error, Debug)]
pub enum RemovePlatformAccountCommandError {
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Another client tried to write to the same aggregate at the same time")]
    Conflict,
    #[error("The organization could not be found")]
    OrganizationNotFound { organization_id: u64 },
    #[error("The platform account could not be found")]
    AccountNotFound { account_id: u64 },
}
