use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub metadata: PageMetadata
}

#[derive(Serialize, ToSchema)]
pub struct PageMetadata {
    pub page: i64,
    #[serde(rename = "pageSize")]
    pub page_size: i64
}
