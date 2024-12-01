use std::sync::Arc;

use crate::core::domain::entity::user::UserCommand;
use crate::core::domain::transaction_manager::TransactionManager;
use crate::core::port::create_user::CreateUserInputBoundary;

pub struct AppState {
    pub transaction_manager: Arc<dyn TransactionManager>,
    pub create_user_repository: Arc<dyn UserCommand>,
    pub user_create_use_case: Arc<dyn CreateUserInputBoundary>,
}
