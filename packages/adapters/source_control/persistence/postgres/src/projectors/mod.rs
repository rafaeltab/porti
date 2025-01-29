use async_trait::async_trait;

pub mod organization;

#[async_trait]
pub trait Projector<TEvent> {
    type Error;
    async fn project(&self, event: TEvent) -> Result<(), Self::Error>;
}
