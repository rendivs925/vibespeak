//! Shared layer - Common utilities & types
//!
//! This layer contains utilities, types, and functions that are
//! used across multiple layers of the application.

pub mod constants;
pub mod utils;

// Re-export commonly used shared items
pub use constants::*;
pub use utils::*;
