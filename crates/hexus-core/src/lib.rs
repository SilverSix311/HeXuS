//! HeXuS Core Library
//!
//! Biometric monitoring system for plural systems.
//! Tracks physiological states across alters.

pub mod error;
pub mod models;
pub mod storage;
pub mod analysis;

pub use error::{HeXuSError, Result};
pub use models::*;
pub use storage::Database;
