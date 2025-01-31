use std::hash::{DefaultHasher, Hash, Hasher};

use crate::entities::{platform::Platform, platform_account::{PlatformAccount, PlatformAccountId}};

#[derive(Default)]
pub struct PlatformAccountFactory {}

impl PlatformAccountFactory {
    pub fn create(&self, name: String, platform_name: String) -> PlatformAccount {
        let mut hasher = DefaultHasher::default();
        name.hash(&mut hasher);

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
