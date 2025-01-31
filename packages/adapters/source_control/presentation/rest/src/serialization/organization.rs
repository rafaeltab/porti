use serde_json::json;
use source_control_domain::entities::{
    organization::Organization, platform_account::PlatformAccount,
};

pub fn organization_to_json(organization: &Organization) -> serde_json::Value {
    let platform_accounts: Vec<serde_json::Value> = organization
        .platform_accounts
        .iter()
        .map(platform_account_to_json)
        .collect();
    json!({
        "id": organization.id.0,
        "name": organization.name,
        "platformAccounts": platform_accounts
    })
}

pub fn platform_account_to_json(platform_account: &PlatformAccount) -> serde_json::Value {
    json!({
        "id": platform_account.id.0,
        "name": platform_account.name,
        "platform": {
            "name": platform_account.platform.name
        }
    })
}
