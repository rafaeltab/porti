use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use shaku::HasProvider;
use source_control_application::module::ApplicationModule;
use source_control_postgres_persistence_adapter::queries::get_organizations::{
    GetOrganizationsQuery, GetOrganizationsQueryError, GetOrganizationsQueryHandler,
};
use tracing::instrument;

use crate::{
    errors::InternalServerError,
    models::{
        organization::PartialOrganizationDto,
        paginated_result::{PageMetadata, PaginatedResult},
    },
};

#[derive(Deserialize, Debug)]
pub struct GetAllArguments {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[utoipa::path(
    responses(
        (status = 200, description = "Organization found successfully", body=PaginatedResult<PartialOrganizationDto>),
        (status = 500, description = "An unexpected issue happened", body=InternalServerError)
    )
)]
#[get("/organizations", name = "organizations")]
#[instrument(skip(module))]
pub async fn get_organizations(
    arguments: web::Query<GetAllArguments>,
    module: web::Data<ApplicationModule>,
) -> HttpResponse {
    let page = arguments.page.unwrap_or(0);
    let page_size = arguments.page_size.unwrap_or(10);
    let query_handler: Box<dyn GetOrganizationsQueryHandler> = module.provide().unwrap();

    let query = GetOrganizationsQuery { page, page_size };

    let result = query_handler.handle(query).await;

    match result {
        Ok(organizations) => {
            let response: PaginatedResult<PartialOrganizationDto> = PaginatedResult {
                items: organizations.iter().map(|o| o.into()).collect(),
                metadata: PageMetadata { page, page_size },
            };
            HttpResponse::Ok().json(response)
        }
        Err(GetOrganizationsQueryError::Connection) => InternalServerError::new(
            "Something went wrong while retreiving the organizations".to_string(),
        )
        .into(),
        Err(GetOrganizationsQueryError::Unexpected) => InternalServerError::new(
            "Something went wrong while retreiving the organizations".to_string(),
        )
        .into(),
    }
}
