use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use source_control_application::queries::get_organization_log::{
    GetOrganizationLogQuery, GetOrganizationLogQueryError, GetOrganizationLogQueryHandler,
};
use tracing::instrument;

use crate::serialization::organization::organization_log_to_json;

#[derive(Deserialize, Debug)]
pub struct GetArguments {
    organization_id: u64,
}

#[instrument]
pub async fn get_organization_log(
    arguments: web::Path<GetArguments>,
    query_handler: web::Data<GetOrganizationLogQueryHandler>,
) -> HttpResponse {
    let command = GetOrganizationLogQuery {
        id: arguments.organization_id,
    };

    let handler = query_handler.get_ref();
    let result = handler.handle(command).await;

    match result {
        Ok(organization_log) => {
            HttpResponse::Ok().json(organization_log_to_json(&organization_log))
        }
        Err(GetOrganizationLogQueryError::NotFound { .. }) => {
            HttpResponse::NotFound().json(json!({
                "message": "Organization not found"
            }))
        }
        Err(GetOrganizationLogQueryError::Connection) => {
            HttpResponse::InternalServerError().json(json!({
                "message": "The server failed to connect to the database"
            }))
        }
        Err(GetOrganizationLogQueryError::Unexpected) => {
            HttpResponse::InternalServerError().json(json!({
                "message": "Something went unexpectedly wrong"
            }))
        }
    }
}
