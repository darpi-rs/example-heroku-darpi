use crate::middleware::{Error, UserRole};
use async_trait::async_trait;
use darpi::RequestParts;
use shaku::{Component, Interface};

#[derive(Component)]
#[shaku(interface = UserExtractor)]
pub struct UserExtractorImpl;

#[async_trait]
impl UserExtractor for UserExtractorImpl {
    async fn extract(&self, _: &RequestParts) -> Result<UserRole, Error> {
        Ok(UserRole::Admin)
    }
}

#[async_trait]
pub trait UserExtractor: Interface {
    async fn extract(&self, p: &RequestParts) -> Result<UserRole, Error>;
}
