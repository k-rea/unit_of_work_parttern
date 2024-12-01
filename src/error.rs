use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Failed to initialize database: {0}")]
    DatabaseInitError(String),
    #[error("Failed to initialize application state: {0}")]
    InitializationError(String),
    #[error("Failed to start server: {0}")]
    ServerError(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}
