//! Settings page component

use crate::domain::entities::{SystemSettings, TailscaleStatus};
use crate::infrastructure::api_client as api;
use crate::presentation::components::{
    Button, ButtonVariant, Card, Header, Icon, IconType, NavBar, StatusBadge,
};
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn Settings() -> impl IntoView {
    let (status, set_status) = create_signal("Loading settings...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());

    let (model_path, set_model_path) =
        create_signal("model/vosk-model-en-us-0.22-lgraph".to_string());
    let (sample_rate, set_sample_rate) = create_signal(16000.0_f32);
    let (enable_tts, set_enable_tts) = create_signal(true);
    let (web_server_port, set_web_server_port) = create_signal(8080_u16);

    let (tailscale_status, set_tailscale_status) = create_signal::<Option<TailscaleStatus>>(None);

    // Load settings on mount
    create_effect(move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            // Load config
            match api::ApiClient::new_default().get_config().await {
                Ok(config) => {
                    // Update form fields from loaded settings
                    set_model_path.set(config.settings.vosk_model_path.clone());
                    set_sample_rate.set(config.settings.sample_rate);
                    set_enable_tts.set(config.settings.enable_tts);
                    set_web_server_port.set(config.settings.web_server_port);
                    set_status.set("Settings loaded".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to load: {}", e));
                    set_status_type.set("error".to_string());
                }
            }

            // Also load Tailscale status
            match api::ApiClient::new_default().get_tailscale_status().await {
                Ok(ts_status) => {
                    set_tailscale_status.set(Some(ts_status));
                }
                Err(_) => {
                    // Set a default status if Tailscale check fails
                    set_tailscale_status.set(Some(TailscaleStatus {
                        enabled: false,
                        connected: false,
                        hostname: None,
                        port: 0,
                        error: Some("Unable to check Tailscale status".to_string()),
                    }));
                }
            }
        });
    });

    let save_settings = move |_| {
        let model = model_path.get();
        let rate = sample_rate.get();
        let tts = enable_tts.get();
        let port = web_server_port.get();

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_status.set("Saving settings...".to_string());
            set_status_type.set("info".to_string());

            // Get current config and update settings
            match api::ApiClient::new_default().get_config().await {
                Ok(mut config) => {
                    config.settings.vosk_model_path = model;
                    config.settings.sample_rate = rate;
                    config.settings.enable_tts = tts;
                    config.settings.web_server_port = port;

                    match api::ApiClient::new_default().update_config(&config).await {
                        Ok(_) => {
                            set_status.set("Settings saved successfully".to_string());
                            set_status_type.set("success".to_string());
                        }
                        Err(e) => {
                            set_status.set(format!("Failed to save: {}", e));
                            set_status_type.set("error".to_string());
                        }
                    }
                }
                Err(e) => {
                    set_status.set(format!("Failed to load config: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50/30">
            <div class="flex flex-col">
                <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                    <StatusBadge message=status status_type=status_type />
                </Header>

                <NavBar _active="settings" />

                <main class="flex-1 px-8 py-10 overflow-y-auto">
                    <div class="max-w-6xl mx-auto">
                        {/* Page Header */}
                        <div class="mb-10">
                            <h1 class="text-3xl font-semibold text-gray-900 tracking-tight mb-3">
                                "Settings"
                            </h1>
                            <p class="text-base text-gray-600 leading-relaxed max-w-2xl">
                                "Configure system preferences and remote access settings."
                            </p>
                        </div>

                        {/* Content Grid */}
                        <div class="space-y-6">
                            <Card title="General Settings">
                                <div class="space-y-6">
                                    <div class="space-y-2">
                                        <label class="text-sm font-semibold text-slate-700">"Model Path"</label>
                                        <input
                                            type="text"
                                            class="w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200"
                                            placeholder="Enter model path"
                                            prop:value=move || model_path.get()
                                            on:input=move |ev| set_model_path.set(event_target_value(&ev))
                                        />
                                    </div>

                                    <div class="space-y-2">
                                        <label class="text-sm font-semibold text-slate-700">"Sample Rate"</label>
                                        <input
                                            type="number"
                                            class="w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200"
                                            placeholder="Enter sample rate"
                                            prop:value=move || sample_rate.get().to_string()
                                            on:input=move |ev| {
                                                if let Ok(rate) = event_target_value(&ev).parse::<f32>() {
                                                    set_sample_rate.set(rate);
                                                }
                                            }
                                        />
                                    </div>

                                    <div class="space-y-2">
                                        <label class="text-sm font-semibold text-slate-700">"Enable TTS"</label>
                                        <select
                                            class="w-full px-3 py-2.5 rounded-xl border border-slate-200 bg-white text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-400 transition-colors"
                                            prop:value=move || if enable_tts.get() { "true" } else { "false" }
                                            on:change=move |ev| {
                                                set_enable_tts.set(event_target_value(&ev) == "true")
                                            }
                                        >
                                            <option value="true">"Yes"</option>
                                            <option value="false">"No"</option>
                                        </select>
                                    </div>

                                    <div class="space-y-2">
                                        <label class="text-sm font-semibold text-slate-700">"Web Server Port"</label>
                                        <input
                                            type="number"
                                            class="w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200"
                                            placeholder="Enter port number"
                                            prop:value=move || web_server_port.get().to_string()
                                            on:input=move |ev| {
                                                if let Ok(port) = event_target_value(&ev).parse::<u16>() {
                                                    set_web_server_port.set(port);
                                                }
                                            }
                                        />
                                    </div>

                                    <div class="pt-4">
                                        <Button variant=ButtonVariant::Primary on:click=save_settings>
                                            <Icon icon_type=IconType::Check class="w-4 h-4 mr-2" />
                                            "Save Settings"
                                        </Button>
                                    </div>
                                </div>
                            </Card>

                            <Card title="Tailscale Remote Access">
                                <div class="space-y-4">
                                    <Show
                                        when=move || tailscale_status.get().is_some()
                                        fallback=|| view! {
                                            <div class="flex items-center gap-3">
                                                <div class="animate-spin h-4 w-4 border-2 border-indigo-600 border-t-transparent rounded-full"></div>
                                                <p class="text-sm text-slate-600">"Checking Tailscale status..."</p>
                                            </div>
                                        }
                                    >
                                        {move || {
                                            let ts = tailscale_status.get().unwrap();
                                            if ts.enabled && ts.connected {
                                                let status_text = format!("Connected - Port: {}", ts.port);
                                                view! {
                                                    <div class="flex items-center gap-3 px-4 py-3 rounded-xl border bg-emerald-50 border-emerald-200 max-w-fit">
                                                        <Icon icon_type=IconType::Check class="w-4 h-4 text-emerald-700" />
                                                        <p class="text-sm font-medium text-emerald-700">{status_text}</p>
                                                    </div>
                                                }
                                            } else {
                                                let status_text = if ts.enabled {
                                                    "Enabled but not connected".to_string()
                                                } else {
                                                    "Tailscale not enabled".to_string()
                                                };
                                                view! {
                                                    <div class="flex items-center gap-3 px-4 py-3 rounded-xl border bg-slate-50 border-slate-200 max-w-fit">
                                                        <Icon icon_type=IconType::X class="w-4 h-4 text-slate-600" />
                                                        <p class="text-sm font-medium text-slate-600">{status_text}</p>
                                                    </div>
                                                }
                                            }
                                        }}
                                    </Show>

                                    <div class="pt-4 border-t border-slate-100">
                                        <h4 class="text-sm font-semibold text-slate-700 mb-3">"Setup Instructions"</h4>
                                        <ol class="text-sm text-slate-600 space-y-2 list-decimal list-inside">
                                            <li>"Install Tailscale on this machine"</li>
                                            <li>"Run "<code class="bg-slate-100 px-1 py-0.5 rounded text-xs font-mono">"sudo tailscale up"</code>" to authenticate"</li>
                                            <li>"Get your IP with "<code class="bg-slate-100 px-1 py-0.5 rounded text-xs font-mono">"tailscale ip -4"</code></li>
                                            <li>"Enable Tailscale above and enter your IP"</li>
                                            <li>
                                                "Access Vibespeak remotely at "
                                                <code class="bg-slate-100 px-1 py-0.5 rounded text-xs font-mono">"http://[TAILSCALE_IP]:8080"</code>
                                            </li>
                                        </ol>
                                    </div>
                                </div>
                            </Card>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    }
}
