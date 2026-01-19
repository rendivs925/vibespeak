//! Vibespeak Leptos CSR Application
//!
//! A modern, reactive web interface for the Vibespeak voice automation system.

pub mod api;
pub mod components;
pub mod pages;
pub mod state;

use axum::Router;
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_router::*;

use pages::*;

#[component]
pub fn App() -> impl IntoView {
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
