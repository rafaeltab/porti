use std::sync::Arc;
use tokio_postgres::{Client, Error};

use async_trait::async_trait;
use source_control_domain::aggregates::organization::OrganizationEvent;
use tracing::{instrument, span, Instrument, Level};

use super::Projector;

pub struct OrganizationProjector {
    pub client: Arc<Client>,
}

#[async_trait]
impl Projector<OrganizationEvent> for OrganizationProjector {
    type Error = Error;

    #[instrument(skip(self), err)]
    async fn project(&self, event: OrganizationEvent) -> Result<(), Error> {
        match event {
            OrganizationEvent::AddPlatformAccount {
                organization_id,
                account,
            } => {
                let id = i64::from_ne_bytes(account.id.0.to_ne_bytes());
                let organization_id = i64::from_ne_bytes(organization_id.0.to_ne_bytes());

                let insert_span = span!(Level::INFO, "insert_platform_account");
                self.client.execute("INSERT INTO \"PlatformAccount\" (id, organization_id, name, platform_name) VALUES ($1, $2, $3, $4);", &[&id, &organization_id, &account.name, &account.platform.name]).instrument(insert_span).await
            }
            OrganizationEvent::RemovePlatformAccount { account_id, .. } => {
                let id = i64::from_ne_bytes(account_id.0.to_ne_bytes());

                let delete_span = span!(Level::INFO, "delete_platform_account");
                self.client
                    .execute("DELETE FROM \"PlatformAccount\" WHERE id = $1;", &[&id])
                    .instrument(delete_span)
                    .await
            }
            OrganizationEvent::CreateOrganizationEvent {
                organization_id,
                name,
            } => {
                let id = i64::from_ne_bytes(organization_id.0.to_ne_bytes());
                let insert_span = span!(Level::INFO, "insert_organization");
                self.client
                    .execute(
                        "INSERT INTO \"Organization\" (id, name)  VALUES ($1, $2);",
                        &[&id, &name],
                    )
                    .instrument(insert_span)
                    .await
            }
        }.map(|_| ())
    }
}
