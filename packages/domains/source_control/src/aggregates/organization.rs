use thiserror::Error;

use crate::entities::{
    organization::{Organization, OrganizationId},
    platform_account::{PlatformAccount, PlatformAccountId},
};

use super::base::{Aggregate, DomainError, DomainEvent};

pub type OrganizationAggregate = Aggregate<OrganizationEvent, Organization>;

impl OrganizationAggregate {
    pub fn add_platform_account(
        &mut self,
        platform_account: PlatformAccount,
    ) -> Result<(), OrganizationError> {
        if self.root.has_account(&platform_account) {
            return Err(OrganizationError::AccountAlreadyAdded {
                account_id: platform_account.id,
                organization_id: self.root.id,
            });
        }

        let event = OrganizationEvent::AddPlatformAccount {
            account: platform_account,
            organization_id: self.root.id,
        };
        self.add_event(event);

        Ok(())
    }

    pub fn remove_platform_account(
        &mut self,
        platform_account: &PlatformAccount,
    ) -> Result<(), OrganizationError> {
        if !self.root.has_account(platform_account) {
            return Err(OrganizationError::AccountNotLinked {
                account_id: platform_account.id,
                organization_id: self.root.id,
            });
        }

        let event = OrganizationEvent::RemovePlatformAccount {
            account_id: platform_account.id,
            organization_id: self.root.id,
        };

        self.add_event(event);

        Ok(())
    }
}

pub enum OrganizationEvent {
    AddPlatformAccount {
        organization_id: OrganizationId,
        account: PlatformAccount,
    },
    RemovePlatformAccount {
        organization_id: OrganizationId,
        account_id: PlatformAccountId,
    },
    CreateOrganizationEvent {
        organization_id: OrganizationId,
        name: String,
    },
}

impl DomainEvent<Organization> for OrganizationEvent {
    fn get_event_type(&self) -> &'static str {
        match self {
            OrganizationEvent::AddPlatformAccount { .. } => {
                "Porti.SourceControl/Aggregates/Organization/AddPlatformAccount"
            }
            OrganizationEvent::RemovePlatformAccount { .. } => {
                "Porti.SourceControl/Aggregates/Organization/RemovePlatformAccount"
            }
            OrganizationEvent::CreateOrganizationEvent { .. } => {
                "Porti.SourceControl/Aggregates/Organization/Create"
            }
        }
    }

    fn apply(&self, aggregate: &mut Organization) {
        match self {
            OrganizationEvent::AddPlatformAccount { account, .. } => {
                aggregate.platform_accounts.push(account.clone());
            }
            OrganizationEvent::RemovePlatformAccount { account_id, .. } => {
                aggregate.platform_accounts.retain(|a| a.id != *account_id)
            }
            OrganizationEvent::CreateOrganizationEvent {
                organization_id,
                name,
            } => {
                aggregate.id = *organization_id;
                aggregate.name = name.clone();
            }
        }
    }

    fn get_aggregate_id(&self) -> &u64 {
        match self {
            OrganizationEvent::AddPlatformAccount {
                organization_id, ..
            } => &organization_id.0,
            OrganizationEvent::RemovePlatformAccount {
                organization_id, ..
            } => &organization_id.0,
            OrganizationEvent::CreateOrganizationEvent {
                organization_id, ..
            } => &organization_id.0,
        }
    }
}

#[derive(Debug, Error)]
pub enum OrganizationError {
    #[error("Account {account_id} is already added to organization {organization_id}")]
    AccountAlreadyAdded {
        account_id: PlatformAccountId,
        organization_id: OrganizationId,
    },
    #[error("Account {account_id} not linked to the aggregate when trying to remove it from organization {organization_id}")]
    AccountNotLinked {
        account_id: PlatformAccountId,
        organization_id: OrganizationId,
    },
}

impl DomainError for OrganizationError {}
