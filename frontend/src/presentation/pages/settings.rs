//! Settings page component

use crate::domain::entities::{TailscaleStatus, SystemSettings};
use crate::infrastructure::api_client as api;
use crate::presentation::components::{Card, Header, NavBar, StatusBadge};
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn Settings() -> impl IntoView {
    let (status, set_status) = create_signal("Loading settings...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());

    let (model_path, set_model_path) = create_signal("model/vosk-model-en-us-0.22-lgraph".to_string());
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
        <div class="container">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar _active="settings" />

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
