use std::fmt::{Debug, Display};

use async_trait::async_trait;
use shaku::Interface;

pub mod organization;

#[async_trait]
pub trait Projector<TEvent>: Interface + Send + Sync {
    async fn project(&self, event: TEvent) -> Result<(), Box<dyn ProjectorError>>;
}

pub trait ProjectorError: Debug + Display + Send + Sync {
    fn get_retryable(&self) -> bool;
}

