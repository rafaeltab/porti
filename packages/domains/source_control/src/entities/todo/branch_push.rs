use chrono::{DateTime, Utc};

use super::commit::Commit;

pub struct BranchPush {
    timestamp: DateTime<Utc>,
    commit: Commit
}
