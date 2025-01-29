use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use source_control_application::queries::get_organization::{
    GetOrganizationQuery, GetOrganizationQueryError, GetOrganizationQueryHandler,
};

#[derive(Deserialize)]
pub struct GetArguments {
    id: u64,
}

pub async fn get_organization(
    arguments: web::Path<GetArguments>,
    query_handler: web::Data<GetOrganizationQueryHandler>,
) -> HttpResponse {
    let command = GetOrganizationQuery { id: arguments.id };

    let handler = query_handler.get_ref();
    let result = handler.handle(command).await;

    match result {
        Ok(organization) => HttpResponse::Ok().json(json!({
            "id": organization.id.0,
            "name": organization.name,
        })),
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
