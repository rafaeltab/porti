use actix_web::{http::StatusCode, HttpRequest, HttpResponse};
use derive_more::Display;
use serde::Serialize;
use utoipa::{PartialSchema, ToSchema};

impl InternalServerError {
    pub fn new<TMessage: Into<String>>(message: TMessage) -> Self {
        InternalServerError {
            title: "Bad Request",
            status: StatusCodeS(StatusCode::INTERNAL_SERVER_ERROR),
            detail: message.into(),
        }
    }
}

impl NotFound {
    pub fn from_request(req: &HttpRequest) -> Self {
        NotFound {
            title: "Not Found",
            status: StatusCodeS(StatusCode::NOT_FOUND),
            detail: "The resource could not be found".to_string(),
            resource: req.full_url().to_string(),
        }
    }

    pub fn from_resource<U, I>(req: &HttpRequest, name: &str, elements: U) -> Self
    where
        U: IntoIterator<Item = I>,
        I: AsRef<str>,
    {
        NotFound {
            title: "Not Found",
            status: StatusCodeS(StatusCode::NOT_FOUND),
            detail: "The resource could not be found".to_string(),
            resource: req.url_for(name, elements).unwrap().into(),
        }
    }
}

impl Conflict {
    pub fn new<TMessage: Into<String>>(message: TMessage) -> Self {
        Conflict {
            title: "Conflict",
            status: StatusCodeS(StatusCode::CONFLICT),
            detail: message.into(),
        }
    }
}

impl BadRequest {
    pub fn new<TMessage: Into<String>>(message: TMessage) -> Self {
        BadRequest {
            title: "BadRequest",
            status: StatusCodeS(StatusCode::BAD_REQUEST),
            detail: message.into(),
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

#[derive(Serialize, Debug, Display, ToSchema)]
#[display("BadRequest")]
pub struct BadRequest {
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

impl From<BadRequest> for HttpResponse {
    fn from(value: BadRequest) -> Self {
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
