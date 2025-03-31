use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub metadata: PageMetadata
}

#[derive(Serialize, ToSchema)]
pub struct PageMetadata {
    pub next: Option<String>,
    pub previous: Option<String>
}
