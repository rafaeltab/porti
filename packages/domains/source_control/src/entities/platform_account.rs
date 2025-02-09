use derive_id::DomainIdentity;

use super::platform::Platform;

#[derive(DomainIdentity)]
pub struct PlatformAccountId(pub u64);

#[derive(Clone, Debug)]
pub struct PlatformAccount {
    pub id: PlatformAccountId,
    pub name: String,
    pub platform: Platform,
}
