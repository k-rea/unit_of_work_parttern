use crate::adapter::store::pg::sqlx_transaction::SqlxTransaction;
use async_trait::async_trait;
use sqlx::PgPool;

use crate::core::domain::transaction::{TransactionError, TransactionWrapper};
use crate::core::domain::transaction_manager::{TransactionManager, TransactionManagerError};
use crate::core::domain::transaction_operation::{BoxedTransactionOperation, TransactionOperationError};

pub struct PgTransactionManager {
    pool: PgPool,
}

impl PgTransactionManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl TransactionManager for PgTransactionManager {
    async fn execute(
        &self,
        operation: Box<dyn BoxedTransactionOperation>,
    ) -> Result<(), TransactionManagerError> {
        let mut transaction: Box<dyn TransactionWrapper> = Box::new(SqlxTransaction::new(
            self.pool.begin().await.map_err(|e| {
                TransactionManagerError::TransactionError(TransactionError::ConnectionError(
                    e.to_string(),
                ))
            })?,
        ));

        match operation.execute(&mut transaction).await {
            Ok(result) => {
                transaction.commit().await?;
                Ok(result)
            }
            Err(e) => {
                if let Err(rollback_err) = transaction.rollback().await {
                    return Err(TransactionManagerError::TransactionError(rollback_err))
                }
                Err(TransactionManagerError::OperationError(e))
            }
        }
    }
}
