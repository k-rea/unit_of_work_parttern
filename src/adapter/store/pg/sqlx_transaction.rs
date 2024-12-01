use crate::core::domain::transaction::{ToSql, TransactionError, TransactionWrapper};
use async_trait::async_trait;
use sqlx::{Postgres, Transaction};

pub struct SqlxTransaction<'t> {
    transaction: Transaction<'t, Postgres>,
}

impl<'a> SqlxTransaction<'a> {
    pub fn new(transaction: Transaction<'a, Postgres>) -> Self {
        Self { transaction }
    }
}

#[async_trait]
impl<'t> TransactionWrapper for SqlxTransaction<'t> {
    async fn execute(
        &mut self,
        query: &str,
        params: Vec<Box<dyn ToSql>>,
    ) -> Result<(), TransactionError> {
        let mut sqlx_query = sqlx::query(query);

        for param in params {
            if let Some(value) = param.as_i32() {
                sqlx_query = sqlx_query.bind(value);
            } else if let Some(value) = param.as_string() {
                sqlx_query = sqlx_query.bind(value);
            } else {
                return Err(TransactionError::BindError(format!(
                    "Unsupported parameter type: {:?}",
                    param
                )));
            }
        }

        sqlx_query
            .execute(&mut *self.transaction)
            .await
            .map_err(|e| {
                TransactionError::ExecutionError(format!(
                    "Failed to execute query: {:?}, error: {:?}",
                    query, e
                ))
            })?;
        Ok(())
    }

    async fn rollback(self: Box<Self>) -> Result<(), TransactionError> {
        self.transaction
            .rollback()
            .await
            .map_err(|e| TransactionError::RollbackError(e.to_string()))
    }

    async fn commit(self: Box<Self>) -> Result<(), TransactionError> {
        self.transaction.commit().await.map_err(|e| {
            TransactionError::CommitError(format!("Failed to commit transaction: {:?}", e))
        })
    }
}
