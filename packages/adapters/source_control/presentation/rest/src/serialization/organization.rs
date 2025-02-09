use serde_json::json;
use source_control_domain::{
    aggregates::{base::DomainEvent, organization::OrganizationEvent},
    entities::{organization::Organization, platform_account::PlatformAccount},
};
use source_control_postgres_persistence_adapter::queries::get_organizations::OrganizationResult;

pub fn organization_result_to_json(organization: &OrganizationResult) -> serde_json::Value {
    json!({
        "id": organization.id.0,
        "name": organization.name,
        "platform_account_count": organization.paltform_account_count,
    })
}

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

pub fn organization_log_to_json(log: &[OrganizationEvent]) -> serde_json::Value {
    let events: Vec<serde_json::Value> = log.iter().map(organization_event_to_json).collect();
    json!(events)
}

fn organization_event_to_json(event: &OrganizationEvent) -> serde_json::Value {
    let event_type = event.get_event_type();
    match event {
        OrganizationEvent::AddPlatformAccount {
            organization_id,
            account,
        } => json!({
            "type":event_type,
            "data": {
                "organization_id": organization_id.0,
                "account": {
                    "id": account.id.0,
                    "name": account.name,
                    "platform": {
                        "name": account.platform.name,
                    }
                }
            }
        }),
        OrganizationEvent::RemovePlatformAccount {
            organization_id,
            account_id,
        } => json!({
            "type":event_type,
            "data": {
                "organization_id": organization_id.0,
                "account_id": account_id.0,
            }
        }),
        OrganizationEvent::CreateOrganizationEvent {
            organization_id,
            name,
        } => json!({
            "type":event_type,
            "data": {
                "organization_id": organization_id.0,
                "name": name,
            }
        }),
    }
}
