use derive_id::DomainIdentity;

use super::platform_account::PlatformAccountId;

#[derive(DomainIdentity)]
pub struct RepositoryId(pub u64);

pub struct Repository {
    pub id: RepositoryId,
    pub name: String,
    pub platform_account: PlatformAccountId,
}
