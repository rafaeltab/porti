#![allow(clippy::duplicated_attributes)]

use actix_web::{
    post,
    web::{self},
};
use actix_web::HttpResponse;
use serde::Deserialize;
use shaku::HasProvider;
use source_control_application::{
    commands::create_organization::{
        CreateOrganizationCommand, CreateOrganizationCommandError, CreateOrganizationCommandHandler,
    },
    module::ApplicationModule,
};
use tracing::instrument;
use utoipa::ToSchema;

use crate::{
    errors::{Conflict, InternalServerError},
    models::organization::OrganizationDto,
};

#[utoipa::path(
    responses(
        (status = 201, description = "Organization created successfully", body=OrganizationDto),
        (status = 409, description = "An organization with the same name already exists", body=Conflict),
        (status = 500, description = "An unexpected issue happened", body=InternalServerError)
    )
)]
#[post("/organizations")]
#[instrument(skip(module))]
pub async fn create_organization(
    arguments: web::Json<CreateArguments>,
    module: web::Data<ApplicationModule>,
) ->  HttpResponse {
    let command = CreateOrganizationCommand {
        name: arguments.name.clone(),
    };

    let command_handler: Box<dyn CreateOrganizationCommandHandler> = module.provide().unwrap();

    let result = command_handler.handle(command).await;

    match result {
        Ok(organization) => {
            let dto: OrganizationDto = (&organization).into();
            HttpResponse::Created().json(dto)
        }
        Err(CreateOrganizationCommandError::Conflict) => {
            Conflict::new("A data conflict happened while creating the organization".to_string()).into()
        }

        Err(CreateOrganizationCommandError::Connection) => {
            InternalServerError::new("Something went wrong while creating organization".to_string()).into()
        }

        Err(CreateOrganizationCommandError::Unexpected) => {
            InternalServerError::new("Something went wrong while creating organization".to_string()).into()
        }
    }
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateArguments {
    name: String,
}
