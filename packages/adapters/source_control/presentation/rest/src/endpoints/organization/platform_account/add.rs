use actix_web::{post, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use shaku::HasProvider;
use source_control_application::{
    commands::add_platform_account::{
        AddPlatformAccountCommand, AddPlatformAccountCommandError, AddPlatformAccountCommandHandler,
    },
    module::ApplicationModule,
};
use tracing::instrument;
use utoipa::ToSchema;

use crate::{
    errors::{Conflict, InternalServerError, NotFound},
    models::organization::OrganizationDto,
};

#[derive(Deserialize, Debug, ToSchema)]
pub struct AddArguments {
    name: String,
    platform: PlatformArgument,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct PlatformArgument {
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct AddPath {
    organization_id: String,
}

#[utoipa::path(
    responses(
        (status = 201, description = "Platform account successfully added", body=OrganizationDto),
        (status = 404, description = "Organization couldn't be found", body=NotFound),
        (status = 409, description = "Platform account already exists on organization", body=Conflict),
        (status = 500, description = "An unexpected issue happened", body=InternalServerError)
    )
)]
#[post("/organizations/{organization_id}/platform-accounts", name = "organization_platform_accounts")]
#[instrument(skip(module, req))]
pub async fn add_platform_account(
    arguments: web::Json<AddArguments>,
    path: web::Path<AddPath>,
    module: web::Data<ApplicationModule>,
    req: HttpRequest,
) -> HttpResponse {
    let parse_result = path.organization_id.parse::<u64>();
    if parse_result.is_err() {
        return HttpResponse::BadRequest().json(json!({
            "message": "Incorrectly formatted organization id in path",
            "field": "organization_id",
            "location": "path"
        }));
    }

    let command_handler: Box<dyn AddPlatformAccountCommandHandler> = module.provide().unwrap();
    let command = AddPlatformAccountCommand {
        organization_id: parse_result.unwrap(),
        name: arguments.name.clone(),
        platform_name: arguments.platform.name.clone(),
    };

    let result = command_handler.handle(command).await;

    match result {
        Ok(organization) => {
            let res: OrganizationDto = (&organization).into();
            HttpResponse::Created().json(res)
        }
        Err(AddPlatformAccountCommandError::Conflict) => {
            Conflict::new("A data conflict happened while adding paltform account".to_string())
                .into()
        }
        Err(AddPlatformAccountCommandError::Connection) => InternalServerError::new(
            "Something went wrong while adding platform account to organization".to_string(),
        )
        .into(),
        Err(AddPlatformAccountCommandError::Unexpected) => InternalServerError::new(
            "Something went wrong while adding platform account to organization".to_string(),
        )
        .into(),
        Err(AddPlatformAccountCommandError::AccountAlreadyAdded) => {
            Conflict::new("Account was already added to this organization".to_string()).into()
        }
        Err(AddPlatformAccountCommandError::NotFound { .. }) => NotFound::from_request(&req).into(),
    }
}
