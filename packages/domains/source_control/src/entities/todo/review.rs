use super::developer::DeveloperId;

pub struct Review {
    pub developer_id: DeveloperId,
    pub stutus: ReviewStatus,
}

pub struct ReviewStatus {
    pub name: String,
    pub accepting: bool,
}
