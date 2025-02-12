use std::fmt::Debug;

use async_trait::async_trait;
use shaku::Interface;
use thiserror::Error;

use crate::{
    aggregates::organization::{OrganizationAggregate, OrganizationEvent},
    entities::organization::{Organization, OrganizationId},
};

#[async_trait]
pub trait OrganizationRepository: Interface {
    async fn get_log(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Box<[OrganizationEvent]>, GetOrganizationLogError>;

    async fn get(
        &self,
        organization_id: OrganizationId,
    ) -> Result<OrganizationAggregate, GetOrganizationError>;

    async fn save(&self, organization: OrganizationAggregate) -> Result<(), SaveOrganizationError>;

    async fn create(&self, name: String) -> Result<Organization, CreateOrganizationError>;
}

#[derive(Error, Debug)]
pub enum GetOrganizationLogError {
    #[error("Organization with {organization_id} not found.")]
    NotFound { organization_id: OrganizationId },
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
}

#[derive(Error, Debug)]
pub enum GetOrganizationError {
    #[error("Organization with {organization_id} not found.")]
    NotFound { organization_id: OrganizationId },
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
}

#[derive(Error, Debug)]
pub enum SaveOrganizationError {
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Another client tried to write to the same aggregate at the same time")]
    Conflict,
}

#[derive(Error, Debug)]
pub enum CreateOrganizationError {
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
    #[error("Another client tried to write to the same aggregate at the same time")]
    Conflict,
}
