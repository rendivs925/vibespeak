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

    // Modal states
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (editing_command, set_editing_command) = create_signal::<Option<Command>>(None);

    // Form fields
    let (form_text, set_form_text) = create_signal("".to_string());
    let (form_category, set_form_category) = create_signal("general".to_string());
    let (form_action_type, set_form_action_type) = create_signal("ShellCommand".to_string());
    let (form_action_value, set_form_action_value) = create_signal("".to_string());

    let load_commands = move || {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_status.set("Loading commands...".to_string());
            set_status_type.set("info".to_string());
            match api::ApiClient::new_default().list_commands().await {
                Ok(response) => {
                    let count = response.commands.len();
                    set_commands.set(response.commands);
                    set_status.set(format!("{} commands loaded", count));
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to load: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let create_command = move || {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let action_json = match form_action_type.get().as_str() {
                "ShellCommand" => serde_json::json!({"ShellCommand": form_action_value.get()}),
                "Workflow" => serde_json::json!({"Workflow": form_action_value.get()}),
                "Script" => serde_json::json!({"Script": form_action_value.get()}),
                _ => serde_json::json!({"ShellCommand": form_action_value.get()}),
            };

            let request = api::CreateCommandRequest {
                text: form_text.get(),
                action: action_json,
                category: form_category.get(),
            };

            match api::ApiClient::new_default().create_command(&request).await {
                Ok(_) => {
                    set_show_create_modal.set(false);
                    set_form_text.set("".to_string());
                    set_form_action_value.set("".to_string());
                    load_commands();
                    set_status.set("Command created successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to create command: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let update_command = move |id: String| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let action_json = match form_action_type.get().as_str() {
                "ShellCommand" => serde_json::json!({"ShellCommand": form_action_value.get()}),
                "Workflow" => serde_json::json!({"Workflow": form_action_value.get()}),
                "Script" => serde_json::json!({"Script": form_action_value.get()}),
                _ => serde_json::json!({"ShellCommand": form_action_value.get()}),
            };

            let request = api::UpdateCommandRequest {
                text: Some(form_text.get()),
                action: Some(action_json),
                category: Some(form_category.get()),
                enabled: None,
            };

            match api::ApiClient::new_default()
                .update_command(&id, &request)
                .await
            {
                Ok(_) => {
                    set_show_edit_modal.set(false);
                    set_editing_command.set(None);
                    load_commands();
                    set_status.set("Command updated successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to update command: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let delete_command = move |id: String| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            if !web_sys::window()
                .unwrap()
                .confirm_with_message("Are you sure you want to delete this command?")
                .unwrap_or(false)
            {
                return;
            }

            match api::ApiClient::new_default().delete_command(&id).await {
                Ok(_) => {
                    load_commands();
                    set_status.set("Command deleted successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to delete command: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let start_edit = move |cmd: Command| {
        set_editing_command.set(Some(cmd.clone()));
        set_form_text.set(cmd.text);
        set_form_category.set(cmd.category);

        // Parse action type and value
        if let Some(shell) = cmd.action.get("ShellCommand") {
            if let Some(cmd_str) = shell.as_str() {
                set_form_action_type.set("ShellCommand".to_string());
                set_form_action_value.set(cmd_str.to_string());
            }
        } else if let Some(workflow) = cmd.action.get("Workflow") {
            if let Some(id) = workflow.as_str() {
                set_form_action_type.set("Workflow".to_string());
                set_form_action_value.set(id.to_string());
            }
        } else if let Some(script) = cmd.action.get("Script") {
            if let Some(id) = script.as_str() {
                set_form_action_type.set("Script".to_string());
                set_form_action_value.set(id.to_string());
            }
        }

        set_show_edit_modal.set(true);
    };

    create_effect(move |_| {
        load_commands();
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
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                        <p style="margin: 0; color: #6c757d;">
                            "These voice commands are recognized by the system. Say the voice text to trigger the action."
                        </p>
                        <button
                            class="btn btn-primary"
                            on:click=move |_| {
                                set_form_text.set("".to_string());
                                set_form_category.set("general".to_string());
                                set_form_action_type.set("ShellCommand".to_string());
                                set_form_action_value.set("".to_string());
                                set_show_create_modal.set(true);
                            }
                        >
                            "Add Command"
                        </button>
                    </div>
                    <table style="width: 100%; border-collapse: collapse;">
                        <thead>
                            <tr>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Voice Text"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Action"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Category"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Status"</th>
                                <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <Show
                                when=move || !commands.get().is_empty()
                                fallback=|| view! {
                                    <tr>
                                        <td colspan="5" style="padding: 20px; text-align: center; color: #6c757d;">
                                            "No commands configured yet."
                                        </td>
                                    </tr>
                                }
                            >
                                <For
                                    each=move || commands.get()
                                    key=|cmd| cmd.id.clone()
                                     children=move |cmd| {
                                         let cmd_clone1 = cmd.clone();
                                         let cmd_clone2 = cmd.clone();
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
                                                     {cmd.action_display()}
                                                 </td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{cmd.category.clone()}</td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                     <span class=format!("status {}", enabled_class) style="padding: 4px 8px; font-size: 12px;">
                                                         {enabled_text}
                                                     </span>
                                                 </td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                     <button
                                                         class="btn btn-sm btn-secondary"
                                                         style="margin-right: 5px;"
                                                         on:click=move |_| start_edit(cmd_clone1.clone())
                                                     >
                                                         "Edit"
                                                     </button>
                                                     <button
                                                         class="btn btn-sm btn-danger"
                                                         on:click=move |_| delete_command(cmd_clone2.id.clone())
                                                     >
                                                         "Delete"
                                                     </button>
                                                 </td>
                                             </tr>
                                         }
                                     }
                                />
                            </Show>
                        </tbody>
                    </table>
                </Card>

                // Create Command Modal
                <Show when=move || show_create_modal.get()>
                    <div class="modal" style="display: block; background: rgba(0,0,0,0.5); position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 1000;">
                        <div class="modal-dialog" style="max-width: 500px; margin: 50px auto;">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h5 class="modal-title">"Create New Command"</h5>
                                    <button type="button" class="btn-close" on:click=move |_| set_show_create_modal.set(false)></button>
                                </div>
                                <div class="modal-body">
                                    <form>
                                        <div class="mb-3">
                                            <label class="form-label">"Voice Text"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_text
                                                on:input=move |e| set_form_text.set(event_target_value(&e))
                                                placeholder="What you say to trigger this command"
                                            />
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Category"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_category
                                                on:input=move |e| set_form_category.set(event_target_value(&e))
                                                placeholder="e.g., general, browser, system"
                                            />
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Action Type"</label>
                                            <select
                                                class="form-control"
                                                prop:value=form_action_type
                                                on:change=move |e| set_form_action_type.set(event_target_value(&e))
                                            >
                                                <option value="ShellCommand">"Shell Command"</option>
                                                <option value="Workflow">"Workflow"</option>
                                                <option value="Script">"Script"</option>
                                            </select>
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Action Value"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_action_value
                                                on:input=move |e| set_form_action_value.set(event_target_value(&e))
                                                placeholder=move || match form_action_type.get().as_str() {
                                                    "ShellCommand" => "Command to execute",
                                                    "Workflow" => "Workflow ID",
                                                    "Script" => "Script ID",
                                                    _ => "Action value",
                                                }
                                            />
                                        </div>
                                    </form>
                                </div>
                                <div class="modal-footer">
                                    <button type="button" class="btn btn-secondary" on:click=move |_| set_show_create_modal.set(false)>"Cancel"</button>
                                    <button type="button" class="btn btn-primary" on:click=move |_| create_command()>"Create Command"</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>

                // Edit Command Modal
                <Show when=move || show_edit_modal.get()>
                    <div class="modal" style="display: block; background: rgba(0,0,0,0.5); position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 1000;">
                        <div class="modal-dialog" style="max-width: 500px; margin: 50px auto;">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h5 class="modal-title">"Edit Command"</h5>
                                    <button type="button" class="btn-close" on:click=move |_| set_show_edit_modal.set(false)></button>
                                </div>
                                <div class="modal-body">
                                    <form>
                                        <div class="mb-3">
                                            <label class="form-label">"Voice Text"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_text
                                                on:input=move |e| set_form_text.set(event_target_value(&e))
                                                placeholder="What you say to trigger this command"
                                            />
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Category"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_category
                                                on:input=move |e| set_form_category.set(event_target_value(&e))
                                                placeholder="e.g., general, browser, system"
                                            />
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Action Type"</label>
                                            <select
                                                class="form-control"
                                                prop:value=form_action_type
                                                on:change=move |e| set_form_action_type.set(event_target_value(&e))
                                            >
                                                <option value="ShellCommand">"Shell Command"</option>
                                                <option value="Workflow">"Workflow"</option>
                                                <option value="Script">"Script"</option>
                                            </select>
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Action Value"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_action_value
                                                on:input=move |e| set_form_action_value.set(event_target_value(&e))
                                                placeholder=move || match form_action_type.get().as_str() {
                                                    "ShellCommand" => "Command to execute",
                                                    "Workflow" => "Workflow ID",
                                                    "Script" => "Script ID",
                                                    _ => "Action value",
                                                }
                                            />
                                        </div>
                                    </form>
                                </div>
                                <div class="modal-footer">
                                    <button type="button" class="btn btn-secondary" on:click=move |_| set_show_edit_modal.set(false)>"Cancel"</button>
                                    <button
                                        type="button"
                                        class="btn btn-primary"
                                        on:click=move |_| {
                                            if let Some(cmd) = editing_command.get() {
                                                update_command(cmd.id);
                                            }
                                        }
                                    >
                                        "Update Command"
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
