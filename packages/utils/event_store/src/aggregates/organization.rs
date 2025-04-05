use source_control_domain::{
    aggregates::organization::OrganizationEvent,
    entities::{
        organization::OrganizationId,
        platform::Platform,
        platform_account::{PlatformAccount, PlatformAccountId},
    },
};

use crate::FromJson;

pub struct EventStoreOrganizationEvent(pub OrganizationEvent);

impl FromJson for EventStoreOrganizationEvent {
    fn from_json(value: serde_json::Value, event_type: &str) -> EventStoreOrganizationEvent {
        match event_type {
            "Porti.SourceControl/Aggregates/Organization/AddPlatformAccount/1" => {
                let organization_id = &value["organization_id"].as_u64().expect("Unexpected AddPlatformAccount deserialization failure");
                let account_id = &value["account"]["id"].as_u64().expect("Unexpected AddPlatformAccount deserialization failure");
                let account_name = &value["account"]["name"].as_str().expect("Unexpected AddPlatformAccount deserialization failure");
                let platform_name = &value["account"]["platform"]["name"].as_str().expect("Unexpected AddPlatformAccount deserialization failure");

                EventStoreOrganizationEvent(OrganizationEvent::AddPlatformAccount {
                    organization_id: OrganizationId(*organization_id),
                    account: PlatformAccount {
                        id: PlatformAccountId(*account_id),
                        name: account_name.to_string(),
                        platform: Platform {
                            name: platform_name.to_string(),
                        },
                    },
                })
            }
            "Porti.SourceControl/Aggregates/Organization/RemovePlatformAccount/1" => {
                let organization_id = &value["organization_id"].as_u64().expect("Unexpected RemovePlatformAccount deserialization failure");
                let account_id = &value["account"]["id"].as_u64().expect("Unexpected RemovePlatformAccount deserialization failure");

                EventStoreOrganizationEvent(OrganizationEvent::RemovePlatformAccount {
                    account_id: PlatformAccountId(*account_id),
                    organization_id: OrganizationId(*organization_id),
                })
            }
            "Porti.SourceControl/Aggregates/Organization/Create/1" => {
                let organization_id = &value["organization_id"].as_u64().expect("Unexpected Create Organization deserialization failure");
                let name = &value["name"].as_str().expect("Unexpected Create Organization deserialization failure");

                EventStoreOrganizationEvent(OrganizationEvent::CreateOrganizationEvent {
                    organization_id: OrganizationId(*organization_id),
                    name: name.to_string(),
                })
            }
            _ => panic!("Unexpected event passed to EventStoreOrganizationEvent.from_json"),
        }
    }
}
