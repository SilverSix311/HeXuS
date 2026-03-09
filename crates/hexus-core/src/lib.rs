//! HeXuS Core Library
//!
//! Biometric monitoring system for plural systems.
//! Tracks physiological states across alters.

pub mod error;
pub mod models;
pub mod storage;
pub mod analysis;

// FFI module for mobile platforms (Swift/Kotlin)
pub mod ffi;

pub use error::{HeXuSError, Result};
pub use models::*;
pub use storage::Database;

// Generate uniffi scaffolding for FFI
uniffi::setup_scaffolding!();
