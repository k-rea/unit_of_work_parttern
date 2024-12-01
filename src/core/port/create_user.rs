use async_trait::async_trait;
use thiserror::Error;
use crate::core::domain::command::CommandError;
use crate::core::domain::entity::user::user::{
    CreateUserValidationError, UnvalidatedCreateUserInput,
};
use crate::core::domain::transaction_manager::TransactionManagerError;

#[async_trait]
pub trait CreateUserInputBoundary: Send + Sync {
    async fn execute(
        &self,
        input: UnvalidatedCreateUserInput,
        output_boundary: &mut dyn CreateUserOutputBoundary,
    ) -> Result<(), CreateUserError>;
}

#[derive(Debug, Error)]
pub enum CreateUserError {
    #[error(transparent)]
    ValidationError(#[from] CreateUserValidationError),

    #[error(transparent)]
    CommandError(#[from] CommandError),

    #[error(transparent)]
    TransactionError(#[from] TransactionManagerError),

    #[error("Failed to process output: {0}")]
    OutputError(#[from] CreateUserOutputError),
}

pub trait CreateUserOutputBoundary: Send + Sync {
    fn execute(&mut self, output: i32) -> Result<(), CreateUserOutputError>;
}

#[derive(Debug, Error)]
pub enum CreateUserOutputError {
    #[error("Failed to format response: {0}")]
    FormatError(String),

    #[error("Failed to set output value: {0}")]
    SetOutputError(String),

    #[error("Invalid output state: {0}")]
    InvalidStateError(String),
}
