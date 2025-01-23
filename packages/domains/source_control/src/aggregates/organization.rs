use thiserror::Error;

use crate::entities::{
    organization::{Organization, OrganizationId},
    platform_account::{PlatformAccount, PlatformAccountId},
};

use super::base::{Aggregate, DomainError, DomainEvent};

pub type OrganizationAggregate = Aggregate<OrganizationEvents, Organization>;

impl OrganizationAggregate {
    pub fn add_platform_account(
        &mut self,
        platform_account: PlatformAccount,
    ) -> Result<(), OrganizationErrors> {
        if self.root.has_account(&platform_account) {
            return Err(OrganizationErrors::AccountAlreadyAdded {
                account_id: platform_account.id,
                organization_id: self.root.id,
            });
        }

        let event = OrganizationEvents::AddPlatformAccount {
            account: platform_account,
            organization_id: self.root.id,
        };
        self.add_event(event);

        Ok(())
    }

    pub fn remove_platform_account(
        &mut self,
        platform_account: &PlatformAccount,
    ) -> Result<(), OrganizationErrors> {
        if !self.root.has_account(platform_account) {
            return Err(OrganizationErrors::AccountNotLinked {
                account_id: platform_account.id,
                organization_id: self.root.id,
            });
        }

        let event = OrganizationEvents::RemovePlatformAccount {
            account_id: platform_account.id,
            organization_id: self.root.id,
        };

        self.add_event(event);

        Ok(())
    }
}

pub enum OrganizationEvents {
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

impl DomainEvent<Organization> for OrganizationEvents {
    fn get_event_type(&self) -> &'static str {
        match self {
            OrganizationEvents::AddPlatformAccount { .. } => {
                "Porti.SourceControl/Aggregates/Organization/AddPlatformAccount"
            }
            OrganizationEvents::RemovePlatformAccount { .. } => {
                "Porti.SourceControl/Aggregates/Organization/RemovePlatformAccount"
            }
            OrganizationEvents::CreateOrganizationEvent { .. } => {
                "Porti.SourceControl/Aggregates/Organization/Create"
            }
        }
    }

    fn apply(&self, aggregate: &mut Organization) {
        match self {
            OrganizationEvents::AddPlatformAccount { account, .. } => {
                aggregate.platform_accounts.push(account.clone());
            }
            OrganizationEvents::RemovePlatformAccount { account_id, .. } => {
                aggregate.platform_accounts.retain(|a| a.id != *account_id)
            }
            OrganizationEvents::CreateOrganizationEvent {
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
            OrganizationEvents::AddPlatformAccount {
                organization_id, ..
            } => &organization_id.0,
            OrganizationEvents::RemovePlatformAccount {
                organization_id, ..
            } => &organization_id.0,
            OrganizationEvents::CreateOrganizationEvent {
                organization_id, ..
            } => &organization_id.0,
        }
    }
}

#[derive(Debug, Error)]
pub enum OrganizationErrors {
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

impl DomainError for OrganizationErrors {}
