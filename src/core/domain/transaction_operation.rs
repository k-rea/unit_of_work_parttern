use async_trait::async_trait;
use thiserror::Error;
use crate::core::domain::command::CommandError;
use crate::core::domain::transaction::{TransactionError, TransactionWrapper};

#[derive(Debug, Error)]
pub enum TransactionOperationError {
    #[error(transparent)]
    TransactionError(#[from] TransactionError),

    #[error(transparent)]
    CommandError(#[from] CommandError), // または他のコマンドのエラー
}

#[async_trait]
pub trait BoxedTransactionOperation: Send + Sync {
    async fn execute(
        &self,
        transaction: &mut Box<dyn TransactionWrapper>,
    ) -> Result<(), TransactionOperationError>;
}
