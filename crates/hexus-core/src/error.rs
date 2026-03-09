//! Error types for HeXuS Core

use thiserror::Error;

/// Result type alias for HeXuS operations
pub type Result<T> = std::result::Result<T, HeXuSError>;

/// Errors that can occur in HeXuS Core
#[derive(Error, Debug)]
pub enum HeXuSError {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Entity not found
    #[error("Entity not found: {entity_type} with id {id}")]
    NotFound {
        entity_type: String,
        id: String,
    },

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid time range
    #[error("Invalid time range: start {start} is after end {end}")]
    InvalidTimeRange {
        start: String,
        end: String,
    },

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl HeXuSError {
    /// Create a not found error
    pub fn not_found(entity_type: impl Into<String>, id: impl Into<String>) -> Self {
        HeXuSError::NotFound {
            entity_type: entity_type.into(),
            id: id.into(),
        }
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        HeXuSError::Validation(msg.into())
    }

    /// Create a generic error
    pub fn other(msg: impl Into<String>) -> Self {
        HeXuSError::Other(msg.into())
    }
}
