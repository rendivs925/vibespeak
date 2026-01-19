//! Dashboard page component

use crate::infrastructure::api_client;
use crate::presentation::components::{Card, Header, NavBar, StatusBadge};
use crate::presentation::state::*;
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (status, set_status) = create_signal("Loading system status...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (loading, set_loading) = create_signal(true);

    // Load config on mount
    create_effect(move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            // Use presentation state hooks
            let config_signal = use_app_config();
            let loading_signal = use_loading();
            let error_signal = use_error();
            let status_signal = use_status_message();

            // Initial data is loaded by the presentation state on app init
            // Just react to the signals
            create_effect(move |_| {
                if let Some(config) = config_signal.get() {
                    set_status.set(format!(
                        "System ready - {} commands loaded",
                        config.commands.len()
                    ));
                    set_status_type.set("success".to_string());
                    set_loading.set(false);
                }

                if let Some(error) = error_signal.get() {
                    set_status.set(format!("Error: {}", error));
                    set_status_type.set("error".to_string());
                    set_loading.set(false);
                }

                if !loading_signal.get() {
                    set_status.set(status_signal.get());
                    set_status_type.set("info".to_string());
                }
            });
        });
        #[cfg(not(target_arch = "wasm32"))]
        leptos::create_effect(move |_| {
            leptos::spawn_local(async move {
                // Use presentation state hooks
                let config_signal = use_app_config();
                let loading_signal = use_loading();
                let error_signal = use_error();
                let status_signal = use_status_message();

                // Initial data is loaded by the presentation state on app init
                // Just react to the signals
                create_effect(move |_| {
                    if let Some(config) = config_signal.get() {
                        set_status.set(format!(
                            "System ready - {} commands loaded",
                            config.commands.len()
                        ));
                        set_status_type.set("success".to_string());
                        set_loading.set(false);
                    }

                    if let Some(error) = error_signal.get() {
                        set_status.set(format!("Error: {}", error));
                        set_status_type.set("error".to_string());
                        set_loading.set(false);
                    }

                    if !loading_signal.get() {
                        set_status.set(status_signal.get());
                        set_status_type.set("info".to_string());
                    }
                });
            });
        });
    });

    let test_voice = move |_| {
        let text = "Hello world. This is a voice test.".to_string();
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_status.set("Generating voice...".to_string());
            let remote_control_service = use_remote_control();
            match remote_control_service.speak_text(text.clone()).await {
                Ok(_) => {
                    set_status.set(format!("Playing voice: \"{}\"", text));
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Voice test failed: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        leptos::create_effect(move |_| {
            leptos::spawn_local(async move {
                set_status.set("Voice test not available in non-WASM environment".to_string());
                set_status_type.set("warning".to_string());
            });
        });
    };

    let refresh_config = move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_loading.set(true);
            // Trigger a reload of initial data
            let state = crate::presentation::state::PresentationState::get();
            state.load_initial_data().await;

            // The presentation state will update the signals automatically
            set_loading.set(false);
        });
        #[cfg(not(target_arch = "wasm32"))]
        leptos::create_effect(move |_| {
            leptos::spawn_local(async move {
                set_loading.set(true);
                // In non-WASM, simulate config refresh
                set_status
                    .set("Configuration refresh not available in non-WASM environment".to_string());
                set_status_type.set("warning".to_string());
                set_loading.set(false);
            });
        });
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50/30">
            <div class="flex flex-col">
                <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                    <StatusBadge message=status status_type=status_type />
                </Header>

                <NavBar active="dashboard" />

                <main class="flex-1 px-8 py-10 overflow-y-auto">
                    <div class="max-w-6xl mx-auto">
                        {/* Page Header */}
                        <div class="mb-10">
                            <h1 class="text-3xl font-semibold text-gray-900 tracking-tight mb-3">
                                "Dashboard"
                            </h1>
                            <p class="text-base text-gray-600 leading-relaxed max-w-2xl">
                                "Monitor system status and perform quick actions."
                            </p>
                        </div>

                        {/* Content Grid */}
                        <div class="space-y-6">
                            <Card title="System Overview">
                                <p class="text-sm text-gray-700 leading-relaxed mb-4">
                                    "Welcome to Vibespeak! Your voice automation system is ready."
                                </p>
                                <div class="space-y-3">
                                    <Show
                                        when=move || !loading.get()
                                        fallback=|| view! {
                                            <div class="flex items-center gap-3">
                                                <div class="animate-spin h-4 w-4 border-2 border-indigo-600 border-t-transparent rounded-full"></div>
                                                <p class="text-sm text-gray-600">"Loading system statistics..."</p>
                                            </div>
                                        }
                                    >
                                        <div class="flex items-center gap-3">
                                            <div class="w-2 h-2 bg-emerald-500 rounded-full"></div>
                                            <p class="text-sm text-gray-700">"System initialized and ready for commands."</p>
                                        </div>
                                    </Show>
                                </div>
                            </Card>

                            <Card title="Quick Actions">
                                <div class="flex flex-wrap gap-3">
                                    <button class="px-4 py-2.5 rounded-xl text-sm font-semibold text-white bg-gradient-to-r from-indigo-600 to-indigo-700 hover:from-indigo-700 hover:to-indigo-800 shadow-md shadow-indigo-200/50 hover:shadow-lg hover:shadow-indigo-300/50 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:ring-offset-2" on:click=test_voice>
                                        "Test Voice"
                                    </button>
                                    <button class="px-4 py-2.5 rounded-xl text-sm font-semibold bg-indigo-50/80 text-indigo-700 border border-indigo-100 hover:bg-indigo-100 hover:border-indigo-300 hover:shadow-sm transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:ring-offset-2" on:click=refresh_config>
                                        "Refresh Config"
                                    </button>
                                </div>
                            </Card>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    }
}
