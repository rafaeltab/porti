use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use source_control_application::commands::create_organization::{
    CreateOrganizationCommand, CreateOrganizationCommandError, CreateOrganizationCommandHandler,
};

use crate::serialization::organization::organization_to_json;

#[derive(Deserialize)]
pub struct CreateArguments {
    name: String,
}

pub async fn create_organization(
    arguments: web::Json<CreateArguments>,
    command_handler: web::Data<CreateOrganizationCommandHandler>,
) -> HttpResponse {
    let command = CreateOrganizationCommand {
        name: arguments.name.clone(),
    };

    let handler = command_handler.get_ref();
    let result = handler.handle(command).await;

    match result {
        Ok(organization) => HttpResponse::Created().json(organization_to_json(&organization)),
        Err(CreateOrganizationCommandError::Conflict) => HttpResponse::Conflict().json(json!({
            "message": "A data conflict happened while creating the organization"
        })),
        Err(CreateOrganizationCommandError::Connection) => HttpResponse::InternalServerError()
            .json(json!({
                "message": "The server failed to connect to the database"
            })),
        Err(CreateOrganizationCommandError::Unexpected) => HttpResponse::InternalServerError()
            .json(json!({
                "message": "Something went unexpectedly wrong"
            })),
    }
}
