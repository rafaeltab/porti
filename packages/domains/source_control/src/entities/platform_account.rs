use derive_id::DomainIdentity;

use super::platform::Platform;

#[derive(DomainIdentity)]
pub struct PlatformAccountId(pub u64);

#[derive(Clone)]
pub struct PlatformAccount {
    pub id: PlatformAccountId,
    pub name: String,
    pub platform: Platform,
}
