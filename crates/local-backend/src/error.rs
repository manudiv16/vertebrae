//! Error types for the local backend.

use vertebrae_core::error::ServiceError;

/// Local-backend-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum LocalError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Route evaluation error: {0}")]
    Route(String),

    #[error("Orchestrator error: {0}")]
    Orchestrator(String),
}

impl From<LocalError> for ServiceError {
    fn from(e: LocalError) -> Self {
        match e {
            LocalError::Database(msg) => ServiceError::ConfigError(msg),
            LocalError::Query(msg) => ServiceError::ConfigError(format!("query error: {}", msg)),
            LocalError::Migration(msg) => ServiceError::ConfigError(format!("migration error: {}", msg)),
            LocalError::Route(msg) => ServiceError::ValidationFailed { message: msg },
            LocalError::Orchestrator(msg) => ServiceError::ConfigError(format!("orchestrator error: {}", msg)),
        }
    }
}

pub type LocalResult<T> = Result<T, LocalError>;
