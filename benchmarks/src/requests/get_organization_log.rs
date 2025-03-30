use std::sync::Arc;

use futures_util::future::BoxFuture;
use reqwest::Client;

use crate::store::Store;

use super::{Request, RequestHandler};

pub struct GetOrganizationLog {
    pub store: Arc<Store>,
    pub root_url: Arc<String>,
}

impl GetOrganizationLog {
    async fn request(&self, client: Client, request_handler: Box<dyn RequestHandler>) {
        let org = self.store.get_random_organization().await;
        if org.id == 0 {
            return;
        }
        let url = format!("{}/organizations/{}/log", self.root_url, org.id);

        let req = client.get(url);
        let req = request_handler.handle_request(req);
        let res = req.send().await;
        let _ = request_handler.handle_response(res);
    }
}

impl Request for GetOrganizationLog {
    fn make_request(
        &self,
        client: Client,
        request_handler: Box<dyn RequestHandler>,
    ) -> BoxFuture<()> {
        Box::pin(self.request(client, request_handler))
    }
}
