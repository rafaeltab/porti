use actix_web::{delete, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use shaku::HasProvider;
use source_control_application::{
    commands::remove_platform_account::{
        RemovePlatformAccountCommand, RemovePlatformAccountCommandError,
        RemovePlatformAccountCommandHandler,
    },
    module::ApplicationModule,
};
use tracing::instrument;

use crate::{
    errors::{BadRequest, Conflict, InternalServerError, NotFound},
    models::organization::OrganizationDto,
};

#[derive(Deserialize, Debug)]
pub struct RemovePath {
    organization_id: String,
    platform_account_id: String,
}

#[utoipa::path(
    responses(
        (status = 201, description = "Platform account successfully removed", body=OrganizationDto),
        (status = 404, description = "Organization or platform couldn't be found", body=NotFound),
        (status = 409, description = "A conflict occurred", body=Conflict),
        (status = 500, description = "An unexpected issue happened", body=InternalServerError)
    )
)]
#[delete("/organizations/{organization_id}/platform-accounts/{platform_account_id}", name = "organization_platform_account")]
#[instrument(skip(module))]
pub async fn remove_platform_account(
    path: web::Path<RemovePath>,
    module: web::Data<ApplicationModule>,
    req: HttpRequest,
) -> HttpResponse {
    let parse_result = path.organization_id.parse::<u64>();
    if parse_result.is_err() {
        return BadRequest::new("Incorrectly formatted organization id in path".to_string()).into();
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
        organization_id: parse_result.clone().unwrap(),
        account_id: platform_account_parse_result.unwrap(),
    };

    let command_handler: Box<dyn RemovePlatformAccountCommandHandler> = module.provide().unwrap();
    let result = command_handler.handle(command).await;

    match result {
        Ok(organization) => {
            let res: OrganizationDto = (&organization).into();
            HttpResponse::Ok().json(res)
        }
        Err(RemovePlatformAccountCommandError::Conflict) => {
            Conflict::new("A data conflict happened while adding paltform account").into()
        }
        Err(RemovePlatformAccountCommandError::Connection) => {
            InternalServerError::new("The server failed to connect to the database").into()
        }
        Err(RemovePlatformAccountCommandError::Unexpected) => {
            InternalServerError::new("Something went unexpectedly wrong").into()
        }
        Err(RemovePlatformAccountCommandError::AccountNotFound { .. }) => {
            NotFound::from_request(&req).into()
        }
        Err(RemovePlatformAccountCommandError::OrganizationNotFound { .. }) => {
            NotFound::from_resource(
                &req,
                "organization",
                &[format!("{}", parse_result.unwrap())],
            )
            .into()
        }
    }
}
