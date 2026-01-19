//! Dashboard page component - Ultra Premium Design

use crate::infrastructure::api_client;
use crate::presentation::components::{
    Button, ButtonVariant, Header, Icon, IconType, NavBar, StatusBadge,
};
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (status, set_status) = create_signal("Loading system status...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (loading, set_loading) = create_signal(true);
    let (commands_count, set_commands_count) = create_signal(0usize);
    let (workflows_count, set_workflows_count) = create_signal(0usize);
    let (scripts_count, set_scripts_count) = create_signal(0usize);

    // Load config on mount
    create_effect(move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            match api_client::ApiClient::new_default().get_config().await {
                Ok(config) => {
                    set_commands_count.set(config.commands.len());
                    set_workflows_count.set(config.workflows.len());
                    set_scripts_count.set(config.scripts.len());
                    set_status.set(format!("System ready - {} commands loaded", config.commands.len()));
                    set_status_type.set("success".to_string());
                    set_loading.set(false);
                }
                Err(e) => {
                    set_status.set(format!("Error: {}", e));
                    set_status_type.set("error".to_string());
                    set_loading.set(false);
                }
            }
        });
    });

    let test_voice = move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_status.set("Generating voice...".to_string());
            set_status_type.set("info".to_string());
            match api_client::ApiClient::new_default().test_voice().await {
                Ok(_) => {
                    set_status.set("Voice test completed successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Voice test failed: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let refresh_config = move |_| {
        set_loading.set(true);
        set_status.set("Refreshing...".to_string());
        set_status_type.set("info".to_string());

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            match api_client::ApiClient::new_default().get_config().await {
                Ok(config) => {
                    set_commands_count.set(config.commands.len());
                    set_workflows_count.set(config.workflows.len());
                    set_scripts_count.set(config.scripts.len());
                    set_status.set(format!("Refreshed - {} commands", config.commands.len()));
                    set_status_type.set("success".to_string());
                    set_loading.set(false);
                }
                Err(e) => {
                    set_status.set(format!("Refresh failed: {}", e));
                    set_status_type.set("error".to_string());
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50/30">
            <div class="flex flex-col">
                <Header title="Vibespeak" subtitle="Voice Automation System">
                    <StatusBadge message=status status_type=status_type />
                </Header>

                <NavBar _active="dashboard" />

                <main class="flex-1 px-4 sm:px-8 py-6 sm:py-10 overflow-y-auto">
                    <div class="max-w-6xl mx-auto">
                        // Page Header
                        <div class="mb-8 sm:mb-10">
                            <p class="text-xs font-semibold tracking-wide uppercase text-indigo-600 mb-2">
                                "Overview"
                            </p>
                            <h1 class="text-2xl sm:text-3xl font-semibold text-gray-900 tracking-tight mb-3">
                                "Dashboard"
                            </h1>
                            <p class="text-sm sm:text-base text-gray-600 leading-relaxed max-w-2xl">
                                "Monitor system status and manage your voice automation setup."
                            </p>
                        </div>

                        // Stats Grid
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 sm:gap-6 mb-8">
                            // Commands Stat
                            <div class="group relative bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 p-5 sm:p-6 transition-all duration-300 hover:shadow-md hover:shadow-slate-100/60 hover:border-slate-200">
                                <div class="flex items-start justify-between">
                                    <div>
                                        <p class="text-xs font-medium text-slate-500 uppercase tracking-wide mb-1">"Commands"</p>
                                        <p class="text-3xl sm:text-4xl font-semibold text-gray-900 tracking-tight">
                                            {move || commands_count.get()}
                                        </p>
                                    </div>
                                    <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-indigo-500 to-indigo-600 flex items-center justify-center shadow-lg shadow-indigo-200/50">
                                        <Icon icon_type=IconType::Commands class="w-6 h-6 text-white" />
                                    </div>
                                </div>
                                <div class="mt-4 flex items-center gap-2">
                                    <span class="inline-flex items-center gap-1 text-xs font-medium text-emerald-600 bg-emerald-50 px-2 py-0.5 rounded-full">
                                        <Icon icon_type=IconType::Check class="w-3 h-3" />
                                        "Active"
                                    </span>
                                </div>
                            </div>

                            // Workflows Stat
                            <div class="group relative bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 p-5 sm:p-6 transition-all duration-300 hover:shadow-md hover:shadow-slate-100/60 hover:border-slate-200">
                                <div class="flex items-start justify-between">
                                    <div>
                                        <p class="text-xs font-medium text-slate-500 uppercase tracking-wide mb-1">"Workflows"</p>
                                        <p class="text-3xl sm:text-4xl font-semibold text-gray-900 tracking-tight">
                                            {move || workflows_count.get()}
                                        </p>
                                    </div>
                                    <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-amber-500 to-amber-600 flex items-center justify-center shadow-lg shadow-amber-200/50">
                                        <Icon icon_type=IconType::Workflows class="w-6 h-6 text-white" />
                                    </div>
                                </div>
                                <div class="mt-4 flex items-center gap-2">
                                    <span class="text-xs text-slate-500">"Automation sequences"</span>
                                </div>
                            </div>

                            // Scripts Stat
                            <div class="group relative bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 p-5 sm:p-6 transition-all duration-300 hover:shadow-md hover:shadow-slate-100/60 hover:border-slate-200">
                                <div class="flex items-start justify-between">
                                    <div>
                                        <p class="text-xs font-medium text-slate-500 uppercase tracking-wide mb-1">"Scripts"</p>
                                        <p class="text-3xl sm:text-4xl font-semibold text-gray-900 tracking-tight">
                                            {move || scripts_count.get()}
                                        </p>
                                    </div>
                                    <div class="w-12 h-12 rounded-xl bg-gradient-to-br from-emerald-500 to-emerald-600 flex items-center justify-center shadow-lg shadow-emerald-200/50">
                                        <Icon icon_type=IconType::Scripts class="w-6 h-6 text-white" />
                                    </div>
                                </div>
                                <div class="mt-4 flex items-center gap-2">
                                    <span class="text-xs text-slate-500">"Custom scripts"</span>
                                </div>
                            </div>

                            // System Status
                            <div class="group relative bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 p-5 sm:p-6 transition-all duration-300 hover:shadow-md hover:shadow-slate-100/60 hover:border-slate-200">
                                <div class="flex items-start justify-between">
                                    <div>
                                        <p class="text-xs font-medium text-slate-500 uppercase tracking-wide mb-1">"Status"</p>
                                        <p class="text-lg font-semibold text-gray-900 tracking-tight">
                                            {move || if loading.get() { "Loading..." } else { "Online" }}
                                        </p>
                                    </div>
                                    <div class=move || format!(
                                        "w-12 h-12 rounded-xl flex items-center justify-center shadow-lg {}",
                                        if loading.get() {
                                            "bg-gradient-to-br from-slate-400 to-slate-500 shadow-slate-200/50"
                                        } else {
                                            "bg-gradient-to-br from-emerald-500 to-emerald-600 shadow-emerald-200/50"
                                        }
                                    )>
                                        <Show
                                            when=move || loading.get()
                                            fallback=|| view! {
                                                <Icon icon_type=IconType::Check class="w-6 h-6 text-white" />
                                            }
                                        >
                                            <div class="animate-spin h-5 w-5 border-2 border-white border-t-transparent rounded-full"></div>
                                        </Show>
                                    </div>
                                </div>
                                <div class="mt-4 flex items-center gap-2">
                                    <Show
                                        when=move || !loading.get()
                                        fallback=|| view! {
                                            <span class="text-xs text-slate-500">"Connecting..."</span>
                                        }
                                    >
                                        <span class="inline-flex items-center gap-1.5 text-xs text-emerald-600">
                                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                                            "System ready"
                                        </span>
                                    </Show>
                                </div>
                            </div>
                        </div>

                        // Quick Actions Card
                        <section class="relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 overflow-hidden mb-6">
                            <div class="px-6 sm:px-7 py-5 border-b border-slate-100 bg-gradient-to-r from-white to-slate-50/30">
                                <div class="flex items-center gap-3">
                                    <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-indigo-600 flex items-center justify-center shadow-md shadow-indigo-200/50">
                                        <Icon icon_type=IconType::Commands class="w-5 h-5 text-white" />
                                    </div>
                                    <div>
                                        <h3 class="text-lg font-semibold text-gray-900 tracking-tight">"Quick Actions"</h3>
                                        <p class="text-xs text-slate-500">"Common operations"</p>
                                    </div>
                                </div>
                            </div>
                            <div class="px-6 sm:px-7 py-6 bg-gradient-to-b from-white to-slate-50/30">
                                <div class="flex flex-wrap gap-3">
                                    <Button variant=ButtonVariant::Primary on:click=test_voice>
                                        <Icon icon_type=IconType::Microphone class="w-4 h-4 mr-2" />
                                        "Test Voice"
                                    </Button>
                                    <Button variant=ButtonVariant::Secondary on:click=refresh_config>
                                        <Icon icon_type=IconType::Refresh class="w-4 h-4 mr-2" />
                                        "Refresh"
                                    </Button>
                                </div>
                            </div>
                        </section>

                        // Feature Highlights
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                            // Voice Commands Feature
                            <a href="/commands" class="group block relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 p-6 transition-all duration-300 hover:shadow-lg hover:shadow-indigo-100/50 hover:border-indigo-200 hover:-translate-y-1">
                                <div class="w-12 h-12 mb-4 rounded-xl bg-gradient-to-br from-indigo-500 to-indigo-600 flex items-center justify-center shadow-md shadow-indigo-200/50 group-hover:scale-110 transition-transform duration-300">
                                    <Icon icon_type=IconType::Microphone class="w-6 h-6 text-white" />
                                </div>
                                <h4 class="text-base font-semibold text-gray-900 mb-2">"Voice Commands"</h4>
                                <p class="text-sm text-slate-600 leading-relaxed mb-4">
                                    "Create custom voice triggers to execute shell commands, workflows, and scripts."
                                </p>
                                <span class="inline-flex items-center text-sm font-medium text-indigo-600 group-hover:text-indigo-700">
                                    "Configure commands"
                                    <Icon icon_type=IconType::ChevronRight class="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" />
                                </span>
                            </a>

                            // Remote Control Feature
                            <a href="/remote" class="group block relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 p-6 transition-all duration-300 hover:shadow-lg hover:shadow-rose-100/50 hover:border-rose-200 hover:-translate-y-1">
                                <div class="w-12 h-12 mb-4 rounded-xl bg-gradient-to-br from-rose-500 to-rose-600 flex items-center justify-center shadow-md shadow-rose-200/50 group-hover:scale-110 transition-transform duration-300">
                                    <Icon icon_type=IconType::Screen class="w-6 h-6 text-white" />
                                </div>
                                <h4 class="text-base font-semibold text-gray-900 mb-2">"Remote Control"</h4>
                                <p class="text-sm text-slate-600 leading-relaxed mb-4">
                                    "Control your desktop remotely with screen sharing, voice dictation, and touch controls."
                                </p>
                                <span class="inline-flex items-center text-sm font-medium text-rose-600 group-hover:text-rose-700">
                                    "Open remote"
                                    <Icon icon_type=IconType::ChevronRight class="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" />
                                </span>
                            </a>

                            // Settings Feature
                            <a href="/settings" class="group block relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 p-6 transition-all duration-300 hover:shadow-lg hover:shadow-slate-100/50 hover:border-slate-300 hover:-translate-y-1">
                                <div class="w-12 h-12 mb-4 rounded-xl bg-gradient-to-br from-slate-600 to-slate-700 flex items-center justify-center shadow-md shadow-slate-200/50 group-hover:scale-110 transition-transform duration-300">
                                    <Icon icon_type=IconType::Settings class="w-6 h-6 text-white" />
                                </div>
                                <h4 class="text-base font-semibold text-gray-900 mb-2">"Settings"</h4>
                                <p class="text-sm text-slate-600 leading-relaxed mb-4">
                                    "Configure voice recognition, TTS, Tailscale integration, and system preferences."
                                </p>
                                <span class="inline-flex items-center text-sm font-medium text-slate-600 group-hover:text-slate-700">
                                    "Open settings"
                                    <Icon icon_type=IconType::ChevronRight class="w-4 h-4 ml-1 group-hover:translate-x-1 transition-transform" />
                                </span>
                            </a>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    }
}
