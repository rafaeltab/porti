use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::Deserialize;
use shaku::HasProvider;
use source_control_application::module::ApplicationModule;
use source_control_domain::entities::organization::OrganizationId;
use source_control_postgres_persistence_adapter::queries::get_organizations::{
    GetOrganizationsQuery, GetOrganizationsQueryError, GetOrganizationsQueryHandler,
};
use tracing::instrument;
use utoipa::IntoParams;

use crate::{
    errors::InternalServerError,
    models::{
        organization::PartialOrganizationDto,
        paginated_result::{PageMetadata, PaginatedResult},
    },
};

#[derive(Deserialize, Debug, IntoParams)]
pub struct GetAllArguments {
    before: Option<u64>,
    after: Option<u64>
}

#[utoipa::path(
    params(
        GetAllArguments
    ),
    responses(
        (status = 200, description = "Organization found successfully", body=PaginatedResult<PartialOrganizationDto>),
        (status = 500, description = "An unexpected issue happened", body=InternalServerError)
    )
)]
#[get("/organizations", name = "organizations")]
#[instrument(skip(module, req))]
pub async fn get_organizations(
    arguments: web::Query<GetAllArguments>,
    module: web::Data<ApplicationModule>,
    req: HttpRequest,
) -> HttpResponse {
    let query_handler: Box<dyn GetOrganizationsQueryHandler> = module.provide().unwrap();

    let query = GetOrganizationsQuery {
        before: arguments.before.map(OrganizationId),
        after: arguments.after.map(OrganizationId),
    };

    let result = query_handler.handle(query).await;

    match result {
        Ok(organizations) => {
            let first_id = organizations.first().map(|x| x.id.to_primitive());
            let last_id = organizations.last().map(|x| x.id.to_primitive());
            let base_url = req.full_url();

            let next = last_id.map(|x| {
                let mut url = base_url.clone();
                url.set_query(Some(&format!("after={}", x)));
                url.into()
            });
            let previous = first_id.map(|x| {
                let mut url = base_url.clone();
                url.set_query(Some(&format!("before={}", x)));
                url.into()
            });

            let response: PaginatedResult<PartialOrganizationDto> = PaginatedResult {
                items: organizations.iter().map(|o| o.into()).collect(),
                metadata: PageMetadata { next, previous },
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
