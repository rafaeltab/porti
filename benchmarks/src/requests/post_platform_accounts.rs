use std::sync::Arc;

use futures_util::future::BoxFuture;
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::{store::Store, NameGenerator};

use super::{Request, RequestHandler};

pub struct PostPlatformAccounts {
    pub store: Arc<Store>,
    pub root_url: Arc<String>,
    pub name_generator: Arc<NameGenerator>,
}

impl PostPlatformAccounts {
    async fn request(&self, client: Client, request_handler: Box<dyn RequestHandler>) {
        let body = json!({
            "name": self.name_generator.generate_name(),
            "platform": {
                "name": "Github"
            }
        });
        let org = self.store.get_random_organization().await;
        if org.id == 0 {
            return;
        }
        let url = format!(
            "{}/organizations/{}/platform-accounts",
            self.root_url, org.id
        );
        let req = client
            .post(url)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&body).unwrap());
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

impl Request for PostPlatformAccounts {
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
