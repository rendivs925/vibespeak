//! Domain layer - Business logic & entities
//!
//! This layer contains the core business entities, value objects,
//! and domain rules that are independent of any external frameworks.

pub mod entities;
pub mod value_objects;

// Re-export commonly used domain types
pub use entities::*;
pub use value_objects::*;
