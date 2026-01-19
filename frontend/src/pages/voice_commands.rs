//! Voice Commands page component

use leptos::*;
use crate::api;
use crate::components::{Header, NavBar, StatusBadge, Card};
use crate::state::CommandInfo;

#[component]
pub fn VoiceCommands() -> impl IntoView {
    let (status, set_status) = create_signal("Loading...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (commands, set_commands) = create_signal::<Vec<CommandInfo>>(vec![]);

    create_effect(move |_| {
        spawn_local(async move {
            match api::get_config().await {
                Ok(config) => {
                    set_commands.set(config.commands);
                    set_status.set("Commands loaded".to_string());
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

                <Card title="Manage Commands">
                    <button class="btn">"Add Command"</button>
                    <table style="width: 100%; border-collapse: collapse; margin-top: 20px;">
                        <thead>
                            <tr>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Voice Text"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Action"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Category"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Show
                                when=move || !commands.get().is_empty()
                                fallback=|| view! {
                                    <tr>
                                        <td colspan="4" style="padding: 12px; text-align: center;">"No commands configured"</td>
                                    </tr>
                                }
                            >
                                <For
                                    each=move || commands.get()
                                    key=|cmd| cmd.id.clone()
                                    children=move |cmd| view! {
                                        <tr>
                                            <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{cmd.text.clone()}</td>
                                            <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{cmd.action.to_string()}</td>
                                            <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{cmd.category.clone()}</td>
                                            <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                <button class="btn btn-secondary" style="margin-right: 5px;">"Edit"</button>
                                                <button class="btn btn-secondary">"Delete"</button>
                                            </td>
                                        </tr>
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
