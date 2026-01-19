//! Presentation layer - UI components & pages
//!
//! This layer contains Leptos components, pages, and UI-specific logic.
//! It depends on the application and domain layers but not on infrastructure.

pub mod components;
pub mod pages;
pub mod state;

// Re-export commonly used presentation types
pub use components::*;
pub use pages::*;
pub use state::*;
