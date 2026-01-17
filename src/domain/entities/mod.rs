pub mod recognition_session;
pub mod voice_command;
pub mod workflow;

pub use recognition_session::*;
pub use voice_command::*;
pub use workflow::*;

// Browser automation entities
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BrowserSession {
    pub browser: BrowserType,
    pub headless: bool,
    pub profile: Option<String>,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum BrowserType {
    Chrome,
    Firefox,
    Safari,
    Edge,
}
