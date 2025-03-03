use std::collections::HashMap;

use actix_web::HttpRequest;

static GET_ORGANIZATIONS: &str = "/organizatons";
static GET_ORGANIZATION: &str = "/organizations/{organization_id}";
static CREATE_ORGANIZATION: &str = "/organizations";
static GET_ORGANIZATION_LOG: &str = "/organizations/{organization_id}/log";
static ADD_PLATFORM_ACCOUNT: &str = "/organizations/{organization_id}/platform-accounts";
static REMOVE_PLATFORM_ACCOUNT: &str = "/organizations/{organization_id}/platform-accounts";
static GET_PLATFORM_ACCOUNT: &str = "/organizations/{organization_id}/platform-accounts/{account_id}";

static ROOT_PATH: &str = "/api/v1";

pub fn resouce_path(path: &str, req: HttpRequest, params: HashMap<& str, &str>) -> String {
    let mut path_section = path.to_string();
    for (key, value) in params {
        path_section = path_section.replace(&format!("{{{}}}", key), value);
    }

    let scheme = req.uri().scheme().unwrap().to_string();
    let authority = req.uri().authority().unwrap().to_string();

    format!("{scheme}://{authority}{ROOT_PATH}{path_section}")
}
