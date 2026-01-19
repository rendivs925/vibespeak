//! Infrastructure layer - External adapters & persistence
//!
//! This layer contains adapters for external systems like APIs,
//! databases, and third-party services.

pub mod api_client;
pub mod local_storage;

// Re-export commonly used infrastructure types
pub use api_client::*;
pub use local_storage::*;
