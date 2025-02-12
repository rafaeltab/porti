use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use shaku_actix::InjectProvided;
use source_control_application::{module::ApplicationModule, queries::get_organization::{
    GetOrganizationQuery, GetOrganizationQueryError, GetOrganizationQueryHandler,
}};
use tracing::instrument;

use crate::serialization::organization::organization_to_json;

#[derive(Deserialize, Debug)]
pub struct GetArguments {
    organization_id: u64,
}

#[instrument(skip(query_handler))]
pub async fn get_organization(
    arguments: web::Path<GetArguments>,
    query_handler: InjectProvided<ApplicationModule, dyn GetOrganizationQueryHandler>,
) -> HttpResponse {
    let command = GetOrganizationQuery {
        id: arguments.organization_id,
    };

    let result = query_handler.handle(command).await;

    match result {
        Ok(organization) => HttpResponse::Ok().json(organization_to_json(&organization)),
        Err(GetOrganizationQueryError::NotFound { .. }) => HttpResponse::NotFound().json(json!({
            "message": "Organization not found"
        })),
        Err(GetOrganizationQueryError::Connection) => {
            HttpResponse::InternalServerError().json(json!({
                "message": "The server failed to connect to the database"
            }))
        }
        Err(GetOrganizationQueryError::Unexpected) => {
            HttpResponse::InternalServerError().json(json!({
                "message": "Something went unexpectedly wrong"
            }))
        }
    }
}
