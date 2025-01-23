use derive_id::DomainIdentity;

use super::platform_account::PlatformAccount;

#[derive(DomainIdentity, Default)]
pub struct OrganizationId(pub u64);

#[derive(Default)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub platform_accounts: Vec<PlatformAccount>,
}

#[allow(dead_code)]
impl Organization {
    pub fn has_account(&self, account: &PlatformAccount) -> bool {
        self.platform_accounts.iter().any(|e| e.id == account.id)
    }
}
