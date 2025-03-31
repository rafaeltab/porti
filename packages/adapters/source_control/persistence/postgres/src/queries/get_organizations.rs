use std::sync::Arc;

use async_trait::async_trait;
use shaku::{Interface, Provider};
use source_control_domain::entities::organization::OrganizationId;
use thiserror::Error;
use tracing::{error, info, span, Instrument, Level};

use crate::provider::PostgresProvider;

const PAGE_SIZE: i64 = 100;

pub struct GetOrganizationsQuery {
    pub before: Option<OrganizationId>,
    pub after: Option<OrganizationId>,
}

pub struct OrganizationResult {
    pub id: OrganizationId,
    pub name: String,
    pub paltform_account_count: i64,
}

#[async_trait]
pub trait GetOrganizationsQueryHandler: Interface {
    async fn handle(
        &self,
        query: GetOrganizationsQuery,
    ) -> Result<Vec<OrganizationResult>, GetOrganizationsQueryError>;
}

#[derive(Provider)]
#[shaku(interface = GetOrganizationsQueryHandler)]
pub struct GetOrganizationsQueryHandlerImpl {
    #[shaku(inject)]
    pub client: Arc<dyn PostgresProvider>,
}

#[async_trait]
impl GetOrganizationsQueryHandler for GetOrganizationsQueryHandlerImpl {
    async fn handle(
        &self,
        query: GetOrganizationsQuery,
    ) -> Result<Vec<OrganizationResult>, GetOrganizationsQueryError> {
        let GetOrganizationsQuery { before, after } = query;
        let span = span!(Level::INFO, "select_organization");
        let client = self.client.get_client().await;
        let result = match (before, after) {
            (None, Some(aft)) => {
                client
                    .query(
                        "select o.*, count(pa.*)
from \"Organization\" o
         left join \"PlatformAccount\" pa ON o.id = pa.organization_id
WHERE o.id > $2
GROUP BY o.id
ORDER BY o.id ASC
LIMIT $1;",
                        &[&PAGE_SIZE, &domain_id_to_dbid(aft.to_primitive())],
                    )
                    .instrument(span)
                    .await
            }
            (Some(bef), None) => {
                client
                    .query(
                        "select o.*, count(pa.*)
from \"Organization\" o
         left join \"PlatformAccount\" pa ON o.id = pa.organization_id
WHERE o.id < $2
GROUP BY o.id
ORDER BY o.id DESC
LIMIT $1;",
                        &[&PAGE_SIZE, &domain_id_to_dbid(bef.to_primitive())],
                    )
                    .instrument(span)
                    .await
            }
            _ => {
                client
                    .query(
                        "select o.*, count(pa.*)
from \"Organization\" o
         left join \"PlatformAccount\" pa ON o.id = pa.organization_id
GROUP BY o.id
ORDER BY o.id ASC
LIMIT $1;",
                        &[&PAGE_SIZE],
                    )
                    .instrument(span)
                    .await
            }
        };

        match result {
            Ok(result) => {
                info!("Successfully queried organizations");
                let vals: Result<Vec<OrganizationResult>, GetOrganizationsQueryError> =
                    result.iter().map(map_row_to_organization_result).collect();

                vals.map(|mut vec| {
                    vec.sort_by_key(|x| domain_id_to_dbid(x.id.to_primitive()));
                    vec
                })
            }
            Err(err) => {
                error!(
                    error = format!("{:?}", err),
                    "Error while querying organizations"
                );
                Err(GetOrganizationsQueryError::Unexpected)
            }
        }
    }
}

fn dbid_to_domain_id(dbid: i64) -> u64 {
    u64::from_ne_bytes(dbid.to_ne_bytes())
}

fn domain_id_to_dbid(domainid: u64) -> i64 {
    i64::from_ne_bytes(domainid.to_ne_bytes())
}

fn map_row_to_organization_result(
    row: &tokio_postgres::Row,
) -> Result<OrganizationResult, GetOrganizationsQueryError> {
    let (raw_id, name, paltform_account_count) = extract_values(row).map_err(|err| {
        error!(
            error = format!("{:?}", err),
            "Error while parsing organizations query response"
        );
        GetOrganizationsQueryError::Unexpected
    })?;

    let id = OrganizationId(dbid_to_domain_id(raw_id));

    Ok(OrganizationResult {
        id,
        name,
        paltform_account_count,
    })
}

fn extract_values(row: &tokio_postgres::Row) -> Result<(i64, String, i64), tokio_postgres::Error> {
    let raw_id = row.try_get("id")?;
    let name = row.try_get("name")?;
    let paltform_account_count = row.try_get("count")?;

    Ok((raw_id, name, paltform_account_count))
}

#[derive(Error, Debug)]
pub enum GetOrganizationsQueryError {
    #[error("Connecting to the server failed")]
    Connection,
    #[error("Unexpected error")]
    Unexpected,
}
