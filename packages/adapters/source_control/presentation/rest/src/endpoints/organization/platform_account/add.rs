use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use source_control_application::commands::add_platform_account::{AddPlatformAccountCommand, AddPlatformAccountCommandError, AddPlatformAccountCommandHandler};
use tracing::instrument;

use crate::serialization::organization::organization_to_json;

#[derive(Deserialize, Debug)]
pub struct AddArguments {
    name: String,
    platform: PlatformArgument,
}

#[derive(Deserialize, Debug)]
pub struct PlatformArgument {
    name: String
}

#[derive(Deserialize, Debug)]
pub struct AddPath {
    organization_id: String
}

#[instrument]
pub async fn add_platform_account(
    arguments: web::Json<AddArguments>,
    path: web::Path<AddPath>,
    command_handler: web::Data<AddPlatformAccountCommandHandler>,
) -> HttpResponse {

    let parse_result = path.organization_id.parse::<u64>();
    if parse_result.is_err() {
        return HttpResponse::BadRequest().json(json!({
            "message": "Incorrectly formatted organization id in path",
            "field": "organization_id",
            "location": "path"
        }));
    }

    let command = AddPlatformAccountCommand {
        organization_id: parse_result.unwrap(),
        name: arguments.name.clone(),
        platform_name: arguments.platform.name.clone(),
    };

    let handler = command_handler.get_ref();
    let result = handler.handle(command).await;

    match result {
        Ok(organization) => HttpResponse::Created().json(organization_to_json(&organization)),
        Err(AddPlatformAccountCommandError::Conflict) => HttpResponse::Conflict().json(json!({
            "message": "A data conflict happened while adding paltform account"
        })),
        Err(AddPlatformAccountCommandError::Connection) => HttpResponse::InternalServerError()
            .json(json!({
                "message": "The server failed to connect to the database"
            })),
        Err(AddPlatformAccountCommandError::Unexpected) => HttpResponse::InternalServerError()
            .json(json!({
                "message": "Something went unexpectedly wrong"
            })),
        Err(AddPlatformAccountCommandError::AccountAlreadyAdded) => HttpResponse::Conflict()
            .json(json!({
                "message": "Account was already added to this organization"
            })),
        Err(AddPlatformAccountCommandError::NotFound { .. }) => HttpResponse::NotFound()
            .json(json!({
                "message": "Organization not found"
            })),
    }
}
