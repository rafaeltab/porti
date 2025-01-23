use derive_id::DomainIdentity;

use super::branch_push::BranchPush;

#[derive(DomainIdentity)]
pub struct BranchId(pub u64);

pub struct Branch {
    pub name: String,
    pub commit_sha: String,
    pub based_on: Option<Box<Branch>>,
    pub pushes: Vec<BranchPush>,
}
