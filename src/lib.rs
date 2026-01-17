// Vibespeak Library - Voice Automation System
//
// This library provides voice-driven automation for Linux workflows.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod shared;

// Re-export commonly used types
pub use shared::{Error, Result, AudioSample, ScriptType, SecurityLevel};
