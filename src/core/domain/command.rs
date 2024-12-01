use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Entity already exists: {entity_type} - {details}")]
    AlreadyExists {
        entity_type: String,
        details: String
    },

    #[error("Entity not found: {entity_type} - {details}")]
    NotFound {
        entity_type: String,
        details: String
    },

    #[error("ConCurrent modification detected: {entity_type}")]
    ConcurrencyError {
        entity_type: String,
    },

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Validation error: {details}")]
    ValidationError {
        details: String,
    },
}

impl CommandError {
    // User向けのヘルパーメソッド
    pub fn user_not_found(id: i32) -> Self {
        CommandError::NotFound {
            entity_type: "User".to_string(),
            details: format!("id: {}", id),
        }
    }

    pub fn user_already_exists(id: i32) -> Self {
        CommandError::AlreadyExists {
            entity_type: "User".to_string(),
            details: format!("id: {}", id),
        }
    }
}