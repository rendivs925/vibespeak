//! Shared constants and configuration values

/// API endpoints
pub const API_BASE: &str = "/api";

/// Application metadata
pub const APP_NAME: &str = "Vibespeak";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_DESCRIPTION: &str = "Voice automation system for Linux";

/// UI constants
pub const SIDEBAR_WIDTH: u32 = 280;
pub const MAX_COMMAND_TEXT_LENGTH: usize = 500;
pub const MAX_ENTITY_ID_LENGTH: usize = 100;
pub const DEFAULT_DEBOUNCE_DELAY: u32 = 300; // ms

/// Voice recognition constants
pub const DEFAULT_SAMPLE_RATE: f32 = 16000.0;
pub const DEFAULT_VOSK_MODEL: &str = "model/vosk-model-small-en-us-0.15";

/// Remote control constants
pub const MOUSE_MOVE_STEP: i32 = 10;
pub const KEYBOARD_SIMULATION_DELAY: u64 = 50; // ms

/// Storage keys
pub const STORAGE_USER_PREFERENCES: &str = "user_preferences";
pub const STORAGE_UI_STATE: &str = "ui_state";
pub const STORAGE_RECENT_COMMANDS: &str = "recent_commands";

/// CSS class names
pub const CSS_LOADING: &str = "loading";
pub const CSS_ERROR: &str = "error";
pub const CSS_SUCCESS: &str = "success";
pub const CSS_WARNING: &str = "warning";
pub const CSS_DISABLED: &str = "disabled";
pub const CSS_ACTIVE: &str = "active";

/// Route paths
pub const ROUTE_DASHBOARD: &str = "/";
pub const ROUTE_REMOTE_CONTROL: &str = "/remote";
pub const ROUTE_VOICE_COMMANDS: &str = "/commands";
pub const ROUTE_WORKFLOWS: &str = "/workflows";
pub const ROUTE_SCRIPTS: &str = "/scripts";
pub const ROUTE_SETTINGS: &str = "/settings";

/// Time constants (in milliseconds)
pub const SECOND_MS: u64 = 1000;
pub const MINUTE_MS: u64 = 60 * SECOND_MS;
pub const HOUR_MS: u64 = 60 * MINUTE_MS;

/// Pagination constants
pub const DEFAULT_PAGE_SIZE: usize = 20;
pub const MAX_PAGE_SIZE: usize = 100;

/// Validation constants
pub const MIN_PORT: u16 = 1024;
pub const MAX_PORT: u16 = 65535;

/// File extensions
pub const SCRIPT_EXTENSIONS: &[&str] = &["sh", "bash", "py", "js", "lua"];

/// Supported languages
pub const SUPPORTED_LANGUAGES: &[&str] = &["en", "es", "fr", "de", "it", "pt", "ru", "zh"];

/// Theme options
pub const THEMES: &[&str] = &["light", "dark", "system"];

/// Security levels
pub const SECURITY_LEVELS: &[&str] = &["trusted", "sandboxed", "restricted"];
