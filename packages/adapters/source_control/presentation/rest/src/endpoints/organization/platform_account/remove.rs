use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use shaku_actix::InjectProvided;
use source_control_application::{commands::remove_platform_account::{
    RemovePlatformAccountCommand, RemovePlatformAccountCommandError,
    RemovePlatformAccountCommandHandler,
}, module::ApplicationModule};
use tracing::instrument;

use crate::serialization::organization::organization_to_json;

#[derive(Deserialize, Debug)]
pub struct RemovePath {
    organization_id: String,
    platform_account_id: String,
}

#[instrument(skip(command_handler))]
pub async fn remove_platform_account(
    path: web::Path<RemovePath>,
    command_handler: InjectProvided<ApplicationModule, dyn RemovePlatformAccountCommandHandler>,
) -> HttpResponse {
    let parse_result = path.organization_id.parse::<u64>();
    if parse_result.is_err() {
        return HttpResponse::BadRequest().json(json!({
            "message": "Incorrectly formatted organization id in path",
            "field": "organization_id",
            "location": "path"
        }));
    }

    let platform_account_parse_result = path.platform_account_id.parse::<u64>();
    if platform_account_parse_result.is_err() {
        return HttpResponse::BadRequest().json(json!({
            "message": "Incorrectly formatted platform account id in path",
            "field": "platform_account_id",
            "location": "path"
        }));
    }

    let command = RemovePlatformAccountCommand {
        organization_id: parse_result.unwrap(),
        account_id: platform_account_parse_result.unwrap(),
    };

    let result = command_handler.handle(command).await;

    match result {
        Ok(organization) => HttpResponse::Created().json(organization_to_json(&organization)),
        Err(RemovePlatformAccountCommandError::Conflict) => HttpResponse::Conflict().json(json!({
            "message": "A data conflict happened while adding paltform account"
        })),
        Err(RemovePlatformAccountCommandError::Connection) => HttpResponse::InternalServerError()
            .json(json!({
                "message": "The server failed to connect to the database"
            })),
        Err(RemovePlatformAccountCommandError::Unexpected) => HttpResponse::InternalServerError()
            .json(json!({
                "message": "Something went unexpectedly wrong"
            })),
        Err(RemovePlatformAccountCommandError::AccountNotFound { .. }) => HttpResponse::NotFound()
            .json(json!({
                "message": "Account not found"
            })),
        Err(RemovePlatformAccountCommandError::OrganizationNotFound { .. }) => {
            HttpResponse::NotFound().json(json!({
                "message": "Organization not found"
            }))
        }
    }
}
