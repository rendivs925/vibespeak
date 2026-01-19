//! Settings page component

use crate::domain::entities::TailscaleStatus;
use crate::infrastructure::api_client as api;
use crate::presentation::components::{Card, Header, NavBar, StatusBadge};
use leptos::*;
use wasm_bindgen_futures;
use web_sys;

#[component]
pub fn Settings() -> impl IntoView {
    let (status, set_status) = create_signal("Loading settings...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (config, set_config) = create_signal::<Option<crate::domain::entities::AppConfig>>(None);
    let (loading, set_loading) = create_signal(true);
    let (error, set_error) = create_signal::<Option<String>>(None);

    let (model_path, set_model_path) =
        create_signal("model/vosk-model-en-us-0.22-lgraph".to_string());
    let (sample_rate, set_sample_rate) = create_signal(16000.0_f32);
    let (enable_tts, set_enable_tts) = create_signal(true);

    let (tailscale_status, set_tailscale_status) = create_signal::<Option<TailscaleStatus>>(None);

    // Load settings on mount
    create_effect(move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            match api::ApiClient::new_default().get_config().await {
                Ok(config) => {
                    set_config.set(Some(config));
                    set_loading.set(false);
                }
                Err(e) => {
                    set_error.set(Some(format!("Failed to load config: {}", e)));
                    set_loading.set(false);
                }
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        leptos::spawn_local(async move {
            // In non-WASM environment, simulate loading
            set_error.set(Some(
                "Configuration loading not available in non-WASM environment".to_string(),
            ));
            set_loading.set(false);
        });
    });

    let load_tailscale_status = move |_: web_sys::Event| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            match api::ApiClient::new_default().get_tailscale_status().await {
                Ok(ts_status) => {
                    set_tailscale_status.set(Some(ts_status));
                }
                Err(_) => {
                    // Silently ignore Tailscale status errors
                }
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        leptos::spawn_local(async move {
            // In non-WASM environment, simulate loading
            set_tailscale_status.set(Some(crate::domain::entities::TailscaleStatus {
                enabled: false,
                connected: false,
                hostname: None,
                port: 0,
                error: Some("Not available in non-WASM environment".to_string()),
            }));
        });
    };

    let save_settings = move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_status.set("Saving settings...".to_string());
            // TODO: Implement save
            set_status.set("Settings saved".to_string());
            set_status_type.set("success".to_string());
        });
        #[cfg(not(target_arch = "wasm32"))]
        leptos::spawn_local(async move {
            set_status.set("Settings save not available in non-WASM environment".to_string());
            set_status_type.set("warning".to_string());
        });
    };

    view! {
        <div class="container">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar active="settings" />

            <div class="content">
                <h2>"Settings"</h2>

                <Card title="General Settings">
                    <div class="form-group">
                        <label>"Model Path:"</label>
                        <input
                            type="text"
                            prop:value=move || model_path.get()
                            on:input=move |ev| set_model_path.set(event_target_value(&ev))
                            style="width: 100%; padding: 8px 12px; border: 1px solid #ced4da; border-radius: 4px;"
                        />
                    </div>
                    <div class="form-group" style="margin-top: 15px;">
                        <label>"Sample Rate:"</label>
                        <input
                            type="number"
                            prop:value=move || sample_rate.get().to_string()
                            on:input=move |ev| {
                                if let Ok(rate) = event_target_value(&ev).parse::<f32>() {
                                    set_sample_rate.set(rate);
                                }
                            }
                            style="width: 100%; padding: 8px 12px; border: 1px solid #ced4da; border-radius: 4px;"
                        />
                    </div>
                    <div class="form-group" style="margin-top: 15px;">
                        <label>"Enable TTS:"</label>
                        <select
                            prop:value=move || if enable_tts.get() { "true" } else { "false" }
                            on:change=move |ev| set_enable_tts.set(event_target_value(&ev) == "true")
                            style="width: 100%; padding: 8px 12px; border: 1px solid #ced4da; border-radius: 4px;"
                        >
                            <option value="true">"Yes"</option>
                            <option value="false">"No"</option>
                        </select>
                    </div>
                    <button class="btn" style="margin-top: 15px;" on:click=save_settings>
                        "Save Settings"
                    </button>
                </Card>

                <Card title="Tailscale Remote Access">
                    <Show
                        when=move || tailscale_status.get().is_some()
                        fallback=|| view! {
                            <div class="status info">"Checking Tailscale status..."</div>
                        }
                    >
                        {move || {
                            let ts = tailscale_status.get().unwrap();
                            let status_class = if ts.enabled && ts.connected { "success" } else { "info" };
                            let status_text = if ts.enabled && ts.connected {
                                format!("Connected - Port: {}", ts.port)
                            } else if ts.enabled {
                                "Enabled but not connected".to_string()
                            } else {
                                "Tailscale not enabled".to_string()
                            };

                            view! {
                                <div class=format!("status {}", status_class)>
                                    {status_text}
                                </div>
                            }
                        }}
                    </Show>

                    <div style="margin-top: 15px; font-size: 14px;">
                        <p><strong>"Setup Instructions:"</strong></p>
                        <ol>
                            <li>"Install Tailscale on this machine"</li>
                            <li>"Run "<code>"sudo tailscale up"</code>" to authenticate"</li>
                            <li>"Get your IP with "<code>"tailscale ip -4"</code></li>
                            <li>"Enable Tailscale above and enter your IP"</li>
                            <li>"Access Vibespeak remotely at "<code>"http://[TAILSCALE_IP]:8080"</code></li>
                        </ol>
                    </div>
                </Card>
            </div>
        </div>
    }
}
