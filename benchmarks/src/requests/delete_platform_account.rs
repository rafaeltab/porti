use std::sync::Arc;

use futures_util::future::BoxFuture;
use reqwest::Client;

use crate::store::Store;

use super::{Request, RequestHandler};

pub struct DeletePlatformAccount {
    pub store: Arc<Store>,
    pub root_url: Arc<String>,
}

impl DeletePlatformAccount {
    async fn request(&self, client: Client, request_handler: Box<dyn RequestHandler>) {
        let (organization_id, platform_account_id) = self.store.get_random_platform_account().await;
        if organization_id == 0 || platform_account_id == 0 {
            return;
        }
        let url = format!(
            "{}/organizations/{}/platform-accounts/{}",
            self.root_url, organization_id, platform_account_id
        );
        let req = client.delete(url);
        let req = request_handler.handle_request(req);
        let res = req.send().await;
        let res = request_handler.handle_response(res);
        if let Ok(response) = &res {
            if response.status().is_success() {
                let _ = self
                    .store
                    .add_action(crate::store::StoreActions::DeletePlatformAccount {
                        organization_id,
                        platform_account_id,
                    })
                    .await;
            }
        }
    }
}

impl Request for DeletePlatformAccount {
    fn make_request(
        &self,
        client: Client,
        request_handler: Box<dyn RequestHandler>,
    ) -> BoxFuture<()> {
        Box::pin(self.request(client, request_handler))
    }
}
