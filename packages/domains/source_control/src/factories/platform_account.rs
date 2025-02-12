use std::hash::{DefaultHasher, Hash, Hasher};

use shaku::{Interface, Provider};

use crate::entities::{
    organization::Organization,
    platform::Platform,
    platform_account::{PlatformAccount, PlatformAccountId},
};

pub trait PlatformAccountFactory: Interface {
    fn create(
        &self,
        name: String,
        platform_name: String,
        organization: &Organization,
    ) -> PlatformAccount;
}

#[derive(Provider)]
#[shaku(interface = PlatformAccountFactory)]
pub struct PlatformAccountFactoryImpl;

impl PlatformAccountFactory for PlatformAccountFactoryImpl {
    fn create(
        &self,
        name: String,
        platform_name: String,
        organization: &Organization,
    ) -> PlatformAccount {
        let mut hasher = DefaultHasher::default();
        name.hash(&mut hasher);
        organization.id.hash(&mut hasher);

        let id = PlatformAccountId(hasher.finish());

        PlatformAccount {
            id,
            name,
            platform: Platform {
                name: platform_name,
            },
        }
    }
}
