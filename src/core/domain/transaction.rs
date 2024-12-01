use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait TransactionWrapper: Send + Sync {
    async fn execute(
        &mut self,
        query: &str,
        params: Vec<Box<dyn ToSql>>,
    ) -> Result<(), TransactionError>;
    async fn rollback(self: Box<Self>) -> Result<(), TransactionError>;
    async fn commit(self: Box<Self>) -> Result<(), TransactionError>;
}

pub trait ToSql: Send + Sync + std::fmt::Debug {
    fn as_i32(&self) -> Option<i32> {
        None
    }
    fn as_string(&self) -> Option<String> {
        None
    }
}

impl ToSql for i32 {
    fn as_i32(&self) -> Option<i32> {
        Some(*self)
    }
}

impl ToSql for String {
    fn as_string(&self) -> Option<String> {
        Some(self.clone())
    }
}

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Failed to execute query: {0}")]
    ExecutionError(String),
    #[error("Failed to commit transaction: {0}")]
    CommitError(String),
    #[error("Failed to rollback transaction: {0}")]
    RollbackError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Parameter binding error: {0}")]
    BindError(String),
}
