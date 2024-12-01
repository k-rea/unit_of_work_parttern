use crate::adapter::config::AppConfig;
use crate::adapter::store::pg::command::user::PgUserRepository;
use crate::adapter::store::pg::transaction_manager::PgTransactionManager;
use crate::adapter::web::app_state::AppState;
use crate::core::use_case::create_user::CreateUserUseCase;
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;

pub struct AppInitializer;

impl AppInitializer {
    pub async fn initialize(config: AppConfig) -> Result<Arc<AppState>, AppInitializerError> {
        let pool = PgPool::connect(&config.db_url())
            .await
            .map_err(|e| AppInitializerError::DatabaseInitError(e.to_string()))?;

        let transaction_manager = Arc::new(PgTransactionManager::new(pool));
        let create_user_repository = Arc::new(PgUserRepository);
        let user_create_use_case = Arc::new(CreateUserUseCase::new(
            create_user_repository.clone(),
            transaction_manager.clone(),
        ));

        Ok(Arc::new(AppState {
            transaction_manager,
            create_user_repository,
            user_create_use_case,
        }))
    }
}

#[derive(Debug, Error)]
pub enum AppInitializerError {
    #[error("Failed to initialize database: {0}")]
    DatabaseInitError(String),
}
