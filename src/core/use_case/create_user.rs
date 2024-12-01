use async_trait::async_trait;
use std::sync::Arc;

use crate::core::domain::entity::user::user::UnvalidatedCreateUserInput;
use crate::core::domain::entity::user::{User, UserCommand};
use crate::core::domain::transaction::TransactionWrapper;
use crate::core::domain::transaction_manager::TransactionManager;
use crate::core::domain::transaction_operation::{
    BoxedTransactionOperation, TransactionOperationError,
};

use crate::core::port::create_user::{
    CreateUserError, CreateUserInputBoundary, CreateUserOutputBoundary,
};

pub struct InsertUserOperation {
    user: User,
    user_repository: Arc<dyn UserCommand>,
}

impl InsertUserOperation {
    pub fn new(user: User, user_repository: Arc<dyn UserCommand>) -> Self {
        Self {
            user,
            user_repository,
        }
    }
}

// BoxedTransactionOperationの実装
#[async_trait]
impl BoxedTransactionOperation for InsertUserOperation {
    async fn execute(
        &self,
        transaction: &mut Box<dyn TransactionWrapper>,
    ) -> Result<(), TransactionOperationError> {
        self.user_repository
            .insert(transaction, self.user.clone())
            .await
            .map_err(|e| TransactionOperationError::CommandError(e))?;
        Ok(())
    }
}

pub struct CreateUserUseCase {
    repository: Arc<dyn UserCommand>,
    transaction_manager: Arc<dyn TransactionManager>,
}

impl CreateUserUseCase {
    pub fn new(
        repository: Arc<dyn UserCommand>,
        transaction_manager: Arc<dyn TransactionManager>,
    ) -> Self {
        Self {
            repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl CreateUserInputBoundary for CreateUserUseCase {
    async fn execute(
        &self,
        input: UnvalidatedCreateUserInput,
        output_boundary: &mut dyn CreateUserOutputBoundary,
    ) -> Result<(), CreateUserError> {
        let user = User::try_from(input)?;
        let id = user.id;
        let operation = Box::new(InsertUserOperation::new(user, self.repository.clone()));
        self.transaction_manager.execute(operation).await?;

        output_boundary.execute(id)?;

        Ok(())
    }
}
