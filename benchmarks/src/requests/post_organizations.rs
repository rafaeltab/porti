use std::sync::Arc;

use futures_util::future::BoxFuture;
use reqwest::Client;
use serde::Deserialize;

use crate::{store::Store, NameGenerator};

use super::{Request, RequestHandler};

pub struct PostOrganizations {
    pub store: Arc<Store>,
    pub root_url: Arc<String>,
    pub name_generator: Arc<NameGenerator>,
}

impl PostOrganizations {
    async fn request(&self, client: Client, request_handler: Box<dyn RequestHandler>) {
        let request_body = format!("{{\"name\":\"{}\"}}", self.name_generator.generate_name());
        let url = format!("{}/organizations", self.root_url);
        let req = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(request_body);
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
                        organization_id: response_json.id,
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

impl Request for PostOrganizations {
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
    id: u64,
    platform_accounts: Vec<ResponsePlatformAccountJson>,
}

#[derive(Deserialize)]
struct ResponsePlatformAccountJson {
    id: u64,
}
