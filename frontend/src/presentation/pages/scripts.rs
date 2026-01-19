//! Scripts page component

use crate::domain::entities::Script;
use crate::infrastructure::api_client as api;
use crate::presentation::components::{Card, Header, NavBar, StatusBadge};
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn Scripts() -> impl IntoView {
    let (status, set_status) = create_signal("Loading...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (scripts, set_scripts) = create_signal::<Vec<Script>>(vec![]);

    // Load scripts on mount
    create_effect(move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            match api::ApiClient::new_default().get_config().await {
                Ok(config) => {
                    set_scripts.set(config.scripts);
                    set_status.set("Scripts loaded".to_string());
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

            <NavBar active="scripts" />

            <div class="content">
                <h2>"Scripts"</h2>

                <Card title="Script Management">
                    <p>"Execute bash, Python, and JavaScript scripts via voice commands."</p>

                    <Show
                        when=move || !scripts.get().is_empty()
                        fallback=|| view! {
                            <div style="margin-top: 20px; padding: 20px; background: #f8f9fa; border-radius: 4px; text-align: center;">
                                <p style="color: #6c757d;">"No scripts configured yet."</p>
                                <p style="font-size: 14px; color: #868e96; margin-top: 10px;">
                                    "Scripts allow you to run custom automation code."
                                </p>
                            </div>
                        }
                    >
                        <table style="width: 100%; border-collapse: collapse; margin-top: 20px;">
                            <thead>
                                <tr>
                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Name"</th>
                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Language"</th>
                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Status"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || scripts.get()
                                    key=|s| s.id.clone()
                                    children=move |s| view! {
                                        <tr>
                                            <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{s.name.clone()}</td>
                                            <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{format!("{:?}", s.language)}</td>
                                            <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                {if s.enabled { "Enabled" } else { "Disabled" }}
                                            </td>
                                        </tr>
                                    }
                                />
                            </tbody>
                        </table>
                    </Show>
                </Card>
            </div>
        </div>
    }
}
