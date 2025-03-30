use futures_util::future::BoxFuture;
use reqwest::{Client, Error, RequestBuilder, Response};

pub mod delete_platform_account;
pub mod get_organization;
pub mod get_organization_log;
pub mod get_organizations;
pub mod openapi;
pub mod post_organizations;
pub mod post_platform_accounts;

pub trait Request {
    fn make_request(
        &self,
        client: Client,
        response_handler: Box<dyn RequestHandler>,
    ) -> BoxFuture<()>;
}

pub trait RequestHandler: Send + Sync {
    fn handle_response(&self, response: Result<Response, Error>) -> Result<Response, Error>;
    fn handle_request(&self, request: RequestBuilder) -> RequestBuilder;
}
