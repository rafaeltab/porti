use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use shaku::HasProvider;
use source_control_application::{
    module::ApplicationModule,
    queries::get_organization::{
        GetOrganizationQuery, GetOrganizationQueryError, GetOrganizationQueryHandler,
    },
};
use tracing::instrument;
use utoipa::ToSchema;

use crate::{
    errors::{InternalServerError, NotFound},
    models::organization::OrganizationDto,
};

#[derive(Deserialize, Debug, ToSchema)]
pub struct GetArguments {
    organization_id: u64,
}

#[utoipa::path(
    responses(
        (status = 200, description = "Organization found successfully", body=OrganizationDto),
        (status = 404, description = "The organization couldn't be found", body=NotFound),
        (status = 500, description = "An unexpected issue happened", body=InternalServerError)
    )
)]
#[get("/organizations/{organization_id}", name = "organization")]
#[instrument(skip(module, req))]
pub async fn get_organization(
    arguments: web::Path<GetArguments>,
    module: web::Data<ApplicationModule>,
    req: HttpRequest,
) -> HttpResponse {
    let query = GetOrganizationQuery {
        id: arguments.organization_id,
    };

    let query_handler: Box<dyn GetOrganizationQueryHandler> = module.provide().unwrap();

    let result = query_handler.handle(query).await;

    match result {
        Ok(organization) => {
            let dto: OrganizationDto = (&organization).into();
            HttpResponse::Ok().json(dto)
        }
        Err(GetOrganizationQueryError::NotFound { .. }) => {
            NotFound::from_request(&req).into()
        }
        Err(GetOrganizationQueryError::Connection) => InternalServerError::new(
            "Something went wrong while retreiving the organization".to_string(),
        )
        .into(),
        Err(GetOrganizationQueryError::Unexpected) => InternalServerError::new(
            "Something went wrong while retreiving the organization".to_string(),
        )
        .into(),
    }
}

