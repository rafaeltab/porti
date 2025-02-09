use std::hash::{DefaultHasher, Hash, Hasher};

use crate::entities::{organization::Organization, platform::Platform, platform_account::{PlatformAccount, PlatformAccountId}};

#[derive(Default, Debug)]
pub struct PlatformAccountFactory {}

impl PlatformAccountFactory {
    pub fn create(&self, name: String, platform_name: String, organization: &Organization) -> PlatformAccount {
        let mut hasher = DefaultHasher::default();
        name.hash(&mut hasher);
        organization.id.hash(&mut hasher);

        let id = PlatformAccountId(hasher.finish());

        PlatformAccount {
            id,
            name,
            platform: Platform {
                name: platform_name
            }
        }
    }
}
