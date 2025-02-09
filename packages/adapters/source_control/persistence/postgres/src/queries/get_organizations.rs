use std::sync::Arc;

use source_control_domain::entities::organization::OrganizationId;
use thiserror::Error;
use tokio_postgres::Client;
use tracing::{error, info};

pub struct GetOrganizationsQuery {
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone)]
pub struct GetOrganizationsQueryHandler {
    pub client: Arc<Client>,
}

pub struct OrganizationResult {
    pub id: OrganizationId,
    pub name: String,
    pub paltform_account_count: i64,
}

impl GetOrganizationsQueryHandler {
    pub async fn handle(
        &self,
        query: GetOrganizationsQuery,
    ) -> Result<Vec<OrganizationResult>, GetOrganizationsQueryError> {
        let limit = query.page_size;
        let offset = query.page * query.page_size;
        let result = self
            .client
            .query(
                "select o.*, count(pa.*)
from \"Organization\" o
         left join \"PlatformAccount\" pa ON o.id = pa.organization_id
GROUP BY o.id
limit $1 offset $2;",
                &[&limit, &offset],
            )
            .await;

        match result {
            Ok(result) => {
                info!("Successfully queried organizations");
                result.iter().map(map_row_to_organization_result).collect()
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
