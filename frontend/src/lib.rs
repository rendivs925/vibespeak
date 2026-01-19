//! Vibespeak Leptos CSR Application
//!
//! A modern, reactive web interface for the Vibespeak voice automation system.
//! Built with clean architecture principles.

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod shared;

// Re-export commonly used types from domain layer
pub use domain::*;

// Re-export application services
pub use application::*;

// Re-export infrastructure adapters
pub use infrastructure::*;

// Re-export presentation components
pub use presentation::*;

use leptos::*;
use leptos_router::*;
use presentation::pages::*;

#[component]
pub fn App() -> impl IntoView {
    // Initialize presentation state with API client
    let api_client = infrastructure::api_client::ApiClient::new_default();
    presentation::state::PresentationState::init(api_client.clone());

    // Load initial data
    let state = presentation::state::PresentationState::get();
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        state.load_initial_data().await;
    });
    #[cfg(not(target_arch = "wasm32"))]
    {
        // In non-WASM environments (like development), use Effect to wrap spawn_local
        let state_clone = state.clone();
        leptos::create_effect(move |_| {
            let state = state_clone.clone();
            leptos::spawn_local(async move {
                state.load_initial_data().await;
            });
        });
    }

    view! {
        <Router>
            <main class="app">
                <Routes>
                    <Route path="/" view=Dashboard />
                    <Route path="/remote" view=RemoteControl />
                    <Route path="/commands" view=VoiceCommands />
                    <Route path="/workflows" view=Workflows />
                    <Route path="/scripts" view=Scripts />
                    <Route path="/settings" view=Settings />
                </Routes>
            </main>
        </Router>
    }
}

/// Get the HTML content for the CSR app
/// This is used by the backend to serve the initial HTML page
pub fn get_html_content() -> &'static str {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Vibespeak - Voice Automation System</title>
</head>
<body>
    <div id="leptos"></div>
    <script type="module">
        import init, { run_app } from '/vibespeak_frontend.js';
        await init();
        run_app();
    </script>
</body>
</html>"#
}

/// Create a simple HTML response for CSR
/// The backend uses this to serve the initial page that loads the WASM
pub fn create_html_response() -> String {
    get_html_content().to_string()
}

// Export the app runner for the HTML to call
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run_app() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    mount_to_body(|| view! { <App /> });
}

// Keep the CSR main function for development
#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    run_app();
}
