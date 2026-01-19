//! Remote Control page component

use crate::infrastructure::api_client as api;
use crate::presentation::components::{Card, Header, NavBar, StatusBadge};
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn RemoteControl() -> impl IntoView {
    let (status, set_status) = create_signal("Ready".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (dictation_text, set_dictation_text) = create_signal(String::new());
    let (dictation_status, set_dictation_status) = create_signal("Dictation ready".to_string());
    let (is_dictating, set_is_dictating) = create_signal(false);
    let (commands_history, set_commands_history) = create_signal::<Vec<String>>(vec![]);

    let execute_command = move |command: String| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_status.set(format!("Processing: \"{}\"", command));
            set_status_type.set("info".to_string());

            match api::ApiClient::new_default()
                .execute_command(&command)
                .await
            {
                Ok(response) => {
                    if response.status == "ok" {
                        set_status.set(format!("Executed: {}", command));
                        set_status_type.set("success".to_string());
                    } else {
                        set_status.set(format!(
                            "Failed: {}",
                            response
                                .error
                                .unwrap_or_else(|| "Unknown error".to_string())
                        ));
                        set_status_type.set("error".to_string());
                    }

                    // Add to history
                    set_commands_history.update(|h| {
                        let time = js_sys::Date::new_0().to_locale_time_string("en-US");
                        h.push(format!(
                            "{}: \"{}\"",
                            time.as_string().unwrap_or_default(),
                            command
                        ));
                    });
                }
                Err(e) => {
                    set_status.set(format!("Error: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        leptos::create_effect(move |_| {
            leptos::spawn_local(async move {
                set_status
                    .set("Command execution not available in non-WASM environment".to_string());
                set_status_type.set("warning".to_string());
            });
        });
    };

    let type_dictation = move |_| {
        let text = dictation_text.get();
        if text.trim().is_empty() {
            set_dictation_status
                .set("No text to insert. Please dictate some text first.".to_string());
            return;
        }

        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_dictation_status.set("Sending text to desktop...".to_string());

            match api::ApiClient::new_default().type_dictation(&text).await {
                Ok(response) => {
                    if response.success {
                        set_dictation_status.set("Text typed into active application!".to_string());
                    } else {
                        set_dictation_status.set(format!(
                            "Typing failed: {}",
                            response
                                .error
                                .unwrap_or_else(|| "Unknown error".to_string())
                        ));
                    }
                }
                Err(e) => {
                    set_dictation_status.set(format!("Error: {}", e));
                }
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        leptos::create_effect(move |_| {
            leptos::spawn_local(async move {
                set_status
                    .set("Command execution not available in non-WASM environment".to_string());
                set_status_type.set("warning".to_string());
            });
        });
    };

    let clear_dictation = move |_| {
        set_dictation_text.set(String::new());
        set_dictation_status.set("Dictation cleared".to_string());
    };

    let touch_commands = vec![
        ("Terminal", "open terminal"),
        ("Browser", "open browser"),
        ("Vol Up", "volume up"),
        ("Vol Down", "volume down"),
        ("Next WS", "workspace next"),
        ("Prev WS", "workspace previous"),
        ("Close Win", "window close"),
        ("Screenshot", "take screenshot"),
    ];

    view! {
        <div class="container">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar active="remote" />

            <div class="content">
                <h2>"Remote Control"</h2>

                <Card title="Voice Control">
                    <p>"Control your desktop with voice commands from mobile"</p>
                    <div class="voice-commands-list" style="margin-top: 15px;">
                        <h4>"Recent Commands"</h4>
                        <div class="commands-history" style="max-height: 200px; overflow-y: auto; background: #f8f9fa; padding: 10px; border-radius: 4px;">
                            <Show
                                when=move || !commands_history.get().is_empty()
                                fallback=|| view! { <div style="color: #6c757d; font-style: italic;">"No commands yet"</div> }
                            >
                                <For
                                    each=move || commands_history.get()
                                    key=|cmd| cmd.clone()
                                    children=move |cmd| view! {
                                        <div style="padding: 5px 0; border-bottom: 1px solid #dee2e6; font-size: 14px;">
                                            {cmd}
                                        </div>
                                    }
                                />
                            </Show>
                        </div>
                    </div>
                </Card>

                <Card title="Touch Controls">
                    <p>"Touch and gesture controls for mobile devices"</p>
                    <div class="touch-controls" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 10px; margin-top: 15px;">
                        <For
                            each=move || touch_commands.clone()
                            key=|(label, _)| label.to_string()
                            children=move |(label, command)| {
                                let cmd = command.to_string();
                                let exec = execute_command.clone();
                                view! {
                                    <button
                                        class="btn touch-btn"
                                        on:click=move |_| exec(cmd.clone())
                                    >
                                        {label}
                                    </button>
                                }
                            }
                        />
                    </div>
                </Card>

                <Card title="Dictation">
                    <p>"Type anywhere without a keyboard using voice dictation"</p>
                    <div style="margin-bottom: 15px;">
                        <div class="status info" style="margin-bottom: 10px;">
                            {move || dictation_status.get()}
                        </div>
                        <input
                            type="text"
                            placeholder="Dictated text will appear here..."
                            style="width: 100%; padding: 8px; border: 1px solid #ced4da; border-radius: 4px;"
                            prop:value=move || dictation_text.get()
                            on:input=move |ev| {
                                set_dictation_text.set(event_target_value(&ev));
                            }
                        />
                    </div>
                    <div style="display: flex; gap: 10px;">
                        <button class="btn" on:click=type_dictation>
                            "Type Text"
                        </button>
                        <button class="btn btn-secondary" on:click=clear_dictation>
                            "Clear"
                        </button>
                    </div>
                    <div style="margin-top: 15px; padding: 10px; background: #e9ecef; border-radius: 4px; font-size: 14px;">
                        <strong>"How to use dictation:"</strong>
                        <ol style="margin: 5px 0; padding-left: 20px;">
                            <li>"Enter or dictate text in the field above"</li>
                            <li><strong>"Switch to your target application"</strong>" (Gmail, VS Code, browser, etc.)"</li>
                            <li>"Click \"Type Text\" - text gets typed exactly like pressing keys!"</li>
                        </ol>
                        <div style="margin-top: 10px; padding: 8px; background: #d1ecf1; border: 1px solid #bee5eb; border-radius: 3px;">
                            <strong>"Keyboard Simulation:"</strong>" Dictation types text globally like a real keyboard - it works in "<strong>"any application"</strong>" that has focus!"
                        </div>
                    </div>
                </Card>
            </div>
        </div>
    }
}
