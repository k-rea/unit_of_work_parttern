use async_trait::async_trait;
use thiserror::Error;

use crate::core::domain::transaction::TransactionError;
use crate::core::domain::transaction_operation::{
    BoxedTransactionOperation, TransactionOperationError,
};

#[async_trait]
pub trait TransactionManager: Send + Sync {
    async fn execute(
        &self,
        operation: Box<dyn BoxedTransactionOperation>,
    ) -> Result<(), TransactionManagerError>;
}

#[derive(Debug, Error)]
pub enum TransactionManagerError {
    #[error("Failed to begin transaction: {0}")]
    BeginError(String),

    #[error(transparent)]
    OperationError(#[from] TransactionOperationError),

    #[error(transparent)]
    TransactionError(#[from] TransactionError),
}
