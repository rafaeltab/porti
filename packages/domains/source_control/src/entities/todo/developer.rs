use derive_id::DomainIdentity;

#[derive(DomainIdentity)]
pub struct DeveloperId(pub u64);

pub struct Developer {
    pub id: DeveloperId,
    pub username: String,
}
