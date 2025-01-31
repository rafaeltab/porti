use std::fmt::Debug;

use async_trait::async_trait;

pub mod organization;

#[async_trait]
pub trait Projector<TEvent> {
    type Error: Debug;
    async fn project(&self, event: TEvent) -> Result<(), Self::Error>;
}
