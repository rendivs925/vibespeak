//! Common utility functions and helpers

use chrono::{DateTime, Utc};

/// Format a timestamp for display
pub fn format_timestamp(timestamp: &str) -> String {
    match DateTime::parse_from_rfc3339(timestamp) {
        Ok(dt) => dt
            .with_timezone(&Utc)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
        Err(_) => timestamp.to_string(),
    }
}

/// Format a duration in milliseconds to a human-readable string
pub fn format_duration(ms: u64) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes % 60, seconds % 60)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

/// Truncate text to a maximum length with ellipsis
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length.saturating_sub(3)])
    }
}

/// Generate a random ID string
pub fn generate_id() -> String {
    // Use simple UUID generation for now (could be improved with crypto API)
    uuid::Uuid::new_v4().to_string()
}

/// Debounce a function call
pub fn debounce<F>(f: F, delay_ms: u32) -> impl FnMut() + 'static
where
    F: Fn() + 'static,
{
    use gloo_timers::callback::Timeout;
    use std::cell::RefCell;
    use std::rc::Rc;

    let f = Rc::new(RefCell::new(Some(f)));
    let timeout = Rc::new(RefCell::new(None::<Timeout>));

    move || {
        let f_clone = f.clone();
        let timeout_clone = timeout.clone();

        let mut timeout_ref = timeout_clone.borrow_mut();
        if let Some(existing_timeout) = timeout_ref.take() {
            existing_timeout.cancel();
        }

        let new_timeout = Timeout::new(delay_ms, move || {
            if let Some(func) = f_clone.borrow_mut().take() {
                func();
            }
        });

        *timeout_ref = Some(new_timeout);
    }
}

/// Check if running in development mode
pub fn is_development() -> bool {
    cfg!(debug_assertions)
}

/// Get the current user agent string
pub fn get_user_agent() -> String {
    web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Safe unwrap with default value
pub fn safe_unwrap<T>(option: Option<T>, default: T) -> T {
    option.unwrap_or(default)
}

/// Safe unwrap with default for references
pub fn safe_unwrap_ref<'a, T>(option: Option<&'a T>, default: &'a T) -> &'a T {
    option.unwrap_or(default)
}
