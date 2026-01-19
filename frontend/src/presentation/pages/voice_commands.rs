//! Voice Commands page component

use crate::domain::entities::Command;
use crate::infrastructure::api_client as api;
use crate::presentation::components::{Card, Header, NavBar, StatusBadge};
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn VoiceCommands() -> impl IntoView {
    let (status, set_status) = create_signal("Loading...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (commands, set_commands) = create_signal::<Vec<Command>>(vec![]);

    create_effect(move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            match api::ApiClient::new_default().get_config().await {
                Ok(config) => {
                    let count = config.commands.len();
                    set_commands.set(config.commands);
                    set_status.set(format!("{} commands loaded", count));
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to load: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    });

    view! {
        <div class="container">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar active="commands" />

            <div class="content">
                <h2>"Voice Commands"</h2>

                <Card title="Available Commands">
                    <p style="margin-bottom: 15px; color: #6c757d;">
                        "These voice commands are recognized by the system. Say the voice text to trigger the action."
                    </p>
                    <table style="width: 100%; border-collapse: collapse;">
                        <thead>
                            <tr>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Voice Text"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Action"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Category"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Show
                                when=move || !commands.get().is_empty()
                                fallback=|| view! {
                                    <tr>
                                        <td colspan="4" style="padding: 20px; text-align: center; color: #6c757d;">
                                            "No commands configured yet."
                                        </td>
                                    </tr>
                                }
                            >
                                <For
                                    each=move || commands.get()
                                    key=|cmd| cmd.id.clone()
                                    children=move |cmd| {
                                        let enabled_class = if cmd.enabled { "success" } else { "warning" };
                                        let enabled_text = if cmd.enabled { "Enabled" } else { "Disabled" };
                                        view! {
                                            <tr>
                                                <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                    <code style="background: #e9ecef; padding: 2px 6px; border-radius: 3px;">
                                                        {cmd.text.clone()}
                                                    </code>
                                                </td>
                                                <td style="padding: 12px; border-bottom: 1px solid #dee2e6; font-size: 13px;">
                                                    {cmd.action.to_string()}
                                                </td>
                                                <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{cmd.category.clone()}</td>
                                                <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                    <span class=format!("status {}", enabled_class) style="padding: 4px 8px; font-size: 12px;">
                                                        {enabled_text}
                                                    </span>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </Show>
                        </tbody>
                    </table>
                </Card>
            </div>
        </div>
    }
}
