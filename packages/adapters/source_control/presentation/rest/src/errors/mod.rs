use actix_web::{http::StatusCode, HttpRequest, HttpResponse};
use derive_more::Display;
use serde::Serialize;
use utoipa::{PartialSchema, ToSchema};

impl InternalServerError {
    pub fn new(message: String) -> Self {
        InternalServerError {
            title: "Bad Request",
            status: StatusCodeS(StatusCode::INTERNAL_SERVER_ERROR),
            detail: message,
        }
    }
}

impl NotFound {
    pub fn new(req: &HttpRequest) -> Self {
        NotFound {
            title: "Not Found",
            status: StatusCodeS(StatusCode::NOT_FOUND),
            detail: "The resource could not be found".to_string(),
            resource: req.full_url().to_string(),
        }
    }
}

impl Conflict {
    pub fn new(message: String) -> Self {
        Conflict {
            title: "Conflict",
            status: StatusCodeS(StatusCode::CONFLICT),
            detail: message.clone(),
        }
    }
}

#[derive(Serialize, Debug, Display, ToSchema)]
#[display("InternalServerError")]
pub struct InternalServerError {
    title: &'static str,
    status: StatusCodeS,
    detail: String,
}

#[derive(Serialize, Debug, Display, ToSchema)]
#[display("NotFound")]
pub struct NotFound {
    title: &'static str,
    status: StatusCodeS,
    detail: String,
    resource: String,
}

#[derive(Serialize, Debug, Display, ToSchema)]
#[display("Conflict")]
pub struct Conflict {
    title: &'static str,
    status: StatusCodeS,
    detail: String,
}

impl From<InternalServerError> for HttpResponse {
    fn from(value: InternalServerError) -> Self {
        HttpResponse::build(value.status.0)
            .content_type("application/problem+json")
            .json(value)
    }
}

impl From<NotFound> for HttpResponse {
    fn from(value: NotFound) -> Self {
        HttpResponse::build(value.status.0)
            .content_type("application/problem+json")
            .json(value)
    }
}

impl From<Conflict> for HttpResponse {
    fn from(value: Conflict) -> Self {
        HttpResponse::build(value.status.0)
            .content_type("application/problem+json")
            .json(value)
    }
}

#[derive(Debug, Display)]
pub struct StatusCodeS(pub StatusCode);

impl Serialize for StatusCodeS {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u16(self.0.as_u16())
    }
}

impl PartialSchema for StatusCodeS {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        u16::schema()
    }
}
impl ToSchema for StatusCodeS {
    fn name() -> std::borrow::Cow<'static, str> {
        u16::name()
    }

    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        u16::schemas(schemas)
    }
}
