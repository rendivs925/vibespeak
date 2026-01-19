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

use axum::Router;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_router::*;
use presentation::pages::*;

#[component]
pub fn App() -> impl IntoView {
    // Initialize presentation state with API client
    let api_client = infrastructure::api_client::ApiClient::new_default();
    presentation::state::PresentationState::init(api_client.clone());

    // Load initial data
    let state = presentation::state::PresentationState::get();
    wasm_bindgen_futures::spawn_local(async move {
        state.load_initial_data().await;
    });

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

/// Create the Leptos CSR router - serves the CSR app instead of static files
pub fn create_leptos_router() -> Router {
    let leptos_options = LeptosOptions::builder()
        .output_name("vibespeak-frontend")
        .site_root(".")
        .build();

    let routes = generate_route_list(App);

    Router::new()
        .leptos_routes(&leptos_options, routes, App)
        .with_state(leptos_options)
}

// Keep the CSR main function for development
#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    mount_to_body(|| view! { <App /> });
}
