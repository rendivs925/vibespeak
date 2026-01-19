//! Dashboard page component

use crate::api;
use crate::components::{Card, Header, NavBar, StatusBadge};
use leptos::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (status, set_status) = create_signal("Loading system status...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (loading, set_loading) = create_signal(true);

    // Load config on mount
    create_effect(move |_| {
        spawn_local(async move {
            match api::get_config().await {
                Ok(config) => {
                    set_status.set(format!(
                        "System ready - {} commands loaded",
                        config.commands.len()
                    ));
                    set_status_type.set("success".to_string());
                    set_loading.set(false);
                }
                Err(e) => {
                    set_status.set(format!("Failed to load config: {}", e));
                    set_status_type.set("error".to_string());
                    set_loading.set(false);
                }
            }
        });
    });

    let test_voice = move |_| {
        let text = "Hello world. This is a voice test.".to_string();
        spawn_local(async move {
            set_status.set("Generating voice...".to_string());
            match api::speak(&text).await {
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
    };

    let refresh_config = move |_| {
        spawn_local(async move {
            set_loading.set(true);
            match api::get_config().await {
                Ok(config) => {
                    set_status.set(format!(
                        "Configuration refreshed - {} commands",
                        config.commands.len()
                    ));
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to refresh: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
            set_loading.set(false);
        });
    };

    view! {
        <div class="container">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar active="dashboard" />

            <div class="content">
                <h2>"Dashboard"</h2>

                <Card title="System Overview">
                    <p>"Welcome to Vibespeak! Your voice automation system is ready."</p>
                    <div class="stats">
                        <Show
                            when=move || !loading.get()
                            fallback=|| view! { <p>"Loading system statistics..."</p> }
                        >
                            <p>"System initialized and ready for commands."</p>
                        </Show>
                    </div>
                </Card>

                <Card title="Quick Actions">
                    <button class="btn" on:click=test_voice>
                        "Test Voice"
                    </button>
                    <button class="btn btn-secondary" on:click=refresh_config>
                        "Refresh Config"
                    </button>
                </Card>
            </div>
        </div>
    }
}
