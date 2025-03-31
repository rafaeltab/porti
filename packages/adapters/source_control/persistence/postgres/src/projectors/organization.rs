use shaku::Provider;
use std::sync::Arc;
use thiserror::Error;

use async_trait::async_trait;
use source_control_domain::aggregates::organization::OrganizationEvent;
use tracing::{instrument, span, Instrument, Level};

use crate::provider::PostgresProvider;

use super::{Projector, ProjectorError};

#[derive(Provider)]
#[shaku(interface = Projector<OrganizationEvent>)]
pub struct OrganizationProjector {
    #[shaku(inject)]
    pub client: Arc<dyn PostgresProvider>,
}

#[derive(Error, Debug)]
enum OrganizationProjectorError {
    #[error("Unknown error")]
    Unexpected(Box<tokio_postgres::Error>),
    #[error("The key already existed in the database")]
    DuplicateKey,
}

impl ProjectorError for OrganizationProjectorError {
    fn get_retryable(&self) -> bool {
        match self {
            OrganizationProjectorError::Unexpected(_) => true,
            OrganizationProjectorError::DuplicateKey => false,
        }
    }
}

#[async_trait]
impl Projector<OrganizationEvent> for OrganizationProjector {
    #[instrument(skip(self), err)]
    async fn project(&self, event: OrganizationEvent) -> Result<(), Box<dyn ProjectorError>> {
        match event {
            OrganizationEvent::AddPlatformAccount {
                organization_id,
                account,
            } => {
                let id = i64::from_ne_bytes(account.id.0.to_ne_bytes());
                let organization_id = i64::from_ne_bytes(organization_id.0.to_ne_bytes());

                let insert_span = span!(Level::INFO, "insert_platform_account");
                let res = self.client.get_client().await.execute("INSERT INTO \"PlatformAccount\" (id, organization_id, name, platform_name) VALUES ($1, $2, $3, $4);", &[&id, &organization_id, &account.name, &account.platform.name]).instrument(insert_span).await;

                match res {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if let Some(db_error) = e.as_db_error() {
                            if db_error.code().code() == "23505" {
                                return Err(Box::new(OrganizationProjectorError::DuplicateKey));
                            }
                        }

                        return Err(Box::new(OrganizationProjectorError::Unexpected(Box::new(
                            e,
                        ))));
                    }
                }
            }
            OrganizationEvent::RemovePlatformAccount { account_id, .. } => {
                let id = i64::from_ne_bytes(account_id.0.to_ne_bytes());

                let delete_span = span!(Level::INFO, "delete_platform_account");
                match self
                    .client
                    .get_client()
                    .await
                    .execute("DELETE FROM \"PlatformAccount\" WHERE id = $1;", &[&id])
                    .instrument(delete_span)
                    .await
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(Box::new(OrganizationProjectorError::Unexpected(Box::new(
                        e,
                    )))),
                }
            }
            OrganizationEvent::CreateOrganizationEvent {
                organization_id,
                name,
            } => {
                let id = i64::from_ne_bytes(organization_id.0.to_ne_bytes());
                let insert_span = span!(Level::INFO, "insert_organization");
                let res = self
                    .client
                    .get_client()
                    .await
                    .execute(
                        "INSERT INTO \"Organization\" (id, name)  VALUES ($1, $2);",
                        &[&id, &name],
                    )
                    .instrument(insert_span)
                    .await;

                match res {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if let Some(db_error) = e.as_db_error() {
                            if db_error.code().code() == "23505" {
                                return Err(Box::new(OrganizationProjectorError::DuplicateKey));
                            }
                        }
                        return Err(Box::new(OrganizationProjectorError::Unexpected(Box::new(
                            e,
                        ))));
                    }
                }
            }
        }
    }
}
