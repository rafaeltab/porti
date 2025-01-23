use derive_id::DomainIdentity;

use super::{repository::RepositoryId, review::Review};

#[derive(DomainIdentity)]
pub struct PullRequestId(pub u64);

pub struct PullRequest {
    pub id: PullRequestId,
    pub repository: RepositoryId,
    pub title: String,
    pub description: String,
    pub reviews: Vec<Review>,
}
