//! Request handlers for the Axum server

pub mod config;
pub mod dictation;
pub mod health;
pub mod remote;
pub mod tts;

pub use config::*;
pub use dictation::*;
pub use health::*;
pub use remote::*;
pub use tts::*;
