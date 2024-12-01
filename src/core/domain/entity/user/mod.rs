pub mod user;

use crate::core::domain::transaction::TransactionWrapper;
use async_trait::async_trait;
use thiserror::Error;
use crate::core::domain::command::CommandError;

#[derive(Debug, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[async_trait]
pub trait UserCommand: Send + Sync {
    async fn insert(
        &self,
        transaction: &mut Box<dyn TransactionWrapper>,
        user: User,
    ) -> Result<(), CommandError>;
}

