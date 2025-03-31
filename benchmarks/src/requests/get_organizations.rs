use std::sync::{Arc, LazyLock};

use futures_util::future::BoxFuture;
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::store::Store;

use super::{Request, RequestHandler};

pub struct GetOrganizations {
    pub store: Arc<Store>,
    pub root_url: Arc<String>,
}

static NEXT_PAGE: LazyLock<RwLock<Option<String>>> = LazyLock::new(|| RwLock::new(None));

impl GetOrganizations {
    async fn request(&self, client: Client, request_handler: Box<dyn RequestHandler>) {
        let mut page_lock = NEXT_PAGE.write().await;

        let page = &*page_lock
            .clone()
            .unwrap_or(format!("{}/organizations", self.root_url));

        let req = client.get(page);
        let req = request_handler.handle_request(req);
        let res = req.send().await;
        let res = request_handler.handle_response(res);
        if let Ok(response) = res {
            if response.status().is_success() {
                let body = response.text().await.unwrap();
                let response_json: Response = serde_json::from_str(&body).unwrap();

                let ids: Vec<u64> = response_json.items.iter().map(|o| o.id).collect();
                *page_lock = response_json.metadata.next;
                let missing_orgs = self.store.get_missing_organizations(ids).await;
                for org in missing_orgs {
                    let _ = self
                        .store
                        .add_action(crate::store::StoreActions::AddOrganization {
                            organization_id: org,
                            platform_accounts: vec![],
                        })
                        .await;
                }
            }
        }
    }
}

impl Request for GetOrganizations {
    fn make_request(
        &self,
        client: Client,
        request_handler: Box<dyn RequestHandler>,
    ) -> BoxFuture<()> {
        Box::pin(self.request(client, request_handler))
    }
}

#[derive(Deserialize)]
struct Response {
    items: Vec<ResponseOrganization>,
    metadata: Metadata,
}

#[derive(Deserialize)]
struct Metadata {
    next: Option<String>,
}

#[derive(Deserialize)]
struct ResponseOrganization {
    id: u64,
}
