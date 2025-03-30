use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use shaku::HasProvider;
use source_control_application::{
    module::ApplicationModule,
    queries::get_organization_log::{
        GetOrganizationLogQuery, GetOrganizationLogQueryError, GetOrganizationLogQueryHandler,
    },
};
use tracing::instrument;

use crate::{
    errors::{InternalServerError, NotFound},
    models::organization_events::OrganizationEventDto,
};

#[derive(Deserialize, Debug)]
pub struct GetArguments {
    organization_id: u64,
}

#[utoipa::path(
    responses(
        (status = 200, description = "Organization found successfully", body=Vec<OrganizationEventDto>),
        (status = 404, description = "The organization couldn't be found", body=NotFound),
        (status = 500, description = "An unexpected issue happened", body=InternalServerError)
    )
)]
#[get("/organizations/{organization_id}/log", name="organization_log")]
#[instrument(skip(module, req))]
pub async fn get_organization_log(
    arguments: web::Path<GetArguments>,
    module: web::Data<ApplicationModule>,
    req: HttpRequest,
) -> HttpResponse {
    let command = GetOrganizationLogQuery {
        id: arguments.organization_id,
    };

    let query_handler: Box<dyn GetOrganizationLogQueryHandler> = module.provide().unwrap();

    let result = query_handler.handle(command).await;

    match result {
        Ok(organization_log) => {
            let res: Vec<OrganizationEventDto> =
                organization_log.iter().map(|e| e.into()).collect();
            HttpResponse::Ok().json(res)
        }
        Err(GetOrganizationLogQueryError::NotFound { .. }) => NotFound::from_request(&req).into(),
        Err(GetOrganizationLogQueryError::Connection) => InternalServerError::new(
            "Something went wrong while retreiving the organization".to_string(),
        )
        .into(),
        Err(GetOrganizationLogQueryError::Unexpected) => InternalServerError::new(
            "Something went wrong while retreiving the organization".to_string(),
        )
        .into(),
    }
}
