//! Vibespeak Leptos CSR Frontend
//!
//! A modern, reactive web interface for the Vibespeak voice automation system.

pub mod api;
pub mod components;
pub mod pages;
pub mod state;

use leptos::*;
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

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

    mount_to_body(|| view! { <App /> });
}
