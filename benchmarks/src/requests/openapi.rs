use std::sync::Arc;

use futures_util::future::BoxFuture;
use reqwest::Client;

use crate::store::Store;

use super::{Request, RequestHandler};

pub struct OpenApi {
    pub store: Arc<Store>,
    pub root_url: Arc<String>,
}

impl OpenApi {
    async fn request(&self, client: Client, request_handler: Box<dyn RequestHandler>) {
        let url = format!("{}/openapi.json", self.root_url);
        let req = client.get(url);
        let req = request_handler.handle_request(req);
        let res = req.send().await;
        let _ = request_handler.handle_response(res);
    }
}

impl Request for OpenApi {
    fn make_request(
        &self,
        client: Client,
        request_handler: Box<dyn RequestHandler>,
    )  -> BoxFuture<()> {
        Box::pin(self.request(client, request_handler))
    }
}
