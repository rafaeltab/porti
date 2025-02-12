use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::{json, Value};
use shaku_actix::InjectProvided;
use source_control_application::module::ApplicationModule;
use source_control_postgres_persistence_adapter::queries::get_organizations::{
    GetOrganizationsQuery, GetOrganizationsQueryError, GetOrganizationsQueryHandler,
};
use tracing::instrument;

use crate::serialization::organization::organization_result_to_json;

#[derive(Deserialize, Debug)]
pub struct GetAllArguments {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[instrument(skip(query_handler))]
pub async fn get_organizations(
    arguments: web::Query<GetAllArguments>,
    query_handler: InjectProvided<ApplicationModule, dyn GetOrganizationsQueryHandler>,
) -> HttpResponse {
    let page = arguments.page.unwrap_or(0);
    let page_size = arguments.page_size.unwrap_or(10);

    let query = GetOrganizationsQuery { page, page_size };

    let result = query_handler.handle(query).await;

    match result {
        Ok(organizations) => {
            let results = organizations
                .iter()
                .map(organization_result_to_json)
                .collect::<Vec<Value>>();
            HttpResponse::Ok().json(json!({
                "items": results,
                "metadata": {
                    "page": page,
                    "page_size": page_size,
                }
            }))
        }
        Err(GetOrganizationsQueryError::Connection) => {
            HttpResponse::InternalServerError().json(json!({
                "message": "The server failed to connect to the database"
            }))
        }
        Err(GetOrganizationsQueryError::Unexpected) => {
            HttpResponse::InternalServerError().json(json!({
                "message": "Something went unexpectedly wrong"
            }))
        }
    }
}
