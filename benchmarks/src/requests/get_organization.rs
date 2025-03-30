use std::sync::Arc;

use futures_util::future::BoxFuture;
use reqwest::Client;
use serde::Deserialize;

use crate::store::Store;

use super::{Request, RequestHandler};

pub struct GetOrganization {
    pub store: Arc<Store>,
    pub root_url: Arc<String>,
}

impl GetOrganization {
    async fn request(&self, client: Client, request_handler: Box<dyn RequestHandler>) {
        let org = self.store.get_random_organization().await;

        if org.id == 0 {
            return;
        }
        let url = format!("{}/organizations/{}", self.root_url, org.id);
        let req = client.get(url);
        let req = request_handler.handle_request(req);
        let res = req.send().await;
        let res = request_handler.handle_response(res);
        if let Ok(response) = res {
            if response.status().is_success() {
                let body = response.text().await.unwrap();
                let response_json: ResponseJson = serde_json::from_str(&body).unwrap();

                let _ = self
                    .store
                    .add_action(crate::store::StoreActions::AddOrganization {
                        organization_id: org.id,
                        platform_accounts: response_json
                            .platform_accounts
                            .iter()
                            .map(|x| x.id)
                            .collect(),
                    })
                    .await;
            }
        }
    }
}

impl Request for GetOrganization {
    fn make_request(
        &self,
        client: Client,
        request_handler: Box<dyn RequestHandler>,
    ) -> BoxFuture<()> {
        Box::pin(self.request(client, request_handler))
    }
}

#[derive(Deserialize)]
struct ResponseJson {
    platform_accounts: Vec<ResponsePlatformAccountJson>,
}

#[derive(Deserialize)]
struct ResponsePlatformAccountJson {
    id: u64,
}
