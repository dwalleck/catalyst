//! Catalyst CLI Library
//!
//! Core library providing types, validation, and helper functions
//! for the Catalyst CLI tool.

pub mod init;
pub mod status;
pub mod types;
pub mod update;
pub mod validation;

// Re-export commonly used types
pub use types::{CatalystError, Platform, Result};
