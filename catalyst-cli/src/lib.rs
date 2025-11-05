//! Catalyst CLI Library
//!
//! Core library providing types, validation, and helper functions
//! for the Catalyst CLI tool.

pub mod types;
pub mod validation;

// Re-export commonly used types
pub use types::{CatalystError, Platform, Result};
