//! Voice Commands page component

use crate::domain::entities::Command;
use crate::infrastructure::api_client as api;
use crate::presentation::components::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Card, DataTable, FormField, Header,
    Input, InputType, Modal, NavBar, Select, StatusBadge, TableCell, TableRow, Textarea,
};
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
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50/30">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar _active="commands" />

            <main class="flex-1 px-8 py-10 overflow-y-auto">
                <div class="max-w-6xl mx-auto">
                    {/* Page Header */}
                    <div class="mb-10">
                        <h1 class="text-3xl font-semibold text-gray-900 tracking-tight mb-3">
                            "Voice Commands"
                        </h1>
                        <p class="text-base text-gray-600 leading-relaxed max-w-2xl">
                            "Manage and create custom voice commands to control your applications."
                        </p>
                    </div>

                    {/* Content Grid */}
                    <div class="space-y-6">
                        <Card title="Available Commands">
                    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
                        <p class="text-gray-600 mb-0">
                            "These voice commands are recognized by the system. Say the voice text to trigger the action."
                        </p>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Medium
                            on:click=move |_| {
                                set_form_text.set("".to_string());
                                set_form_category.set("general".to_string());
                                set_form_action_type.set("ShellCommand".to_string());
                                set_form_action_value.set("".to_string());
                                set_show_create_modal.set(true);
                            }
                        >
                            "Add Command"
                        </Button>
                    </div>
                    <DataTable headers=vec!["Voice Text".to_string(), "Action".to_string(), "Category".to_string(), "Status".to_string(), "Actions".to_string()]>
                        <Show
                            when=move || !commands.get().is_empty()
                            fallback=|| view! {
                                <TableRow>
                                    <TableCell class="text-center text-gray-500 py-8" attr:colspan="5">
                                        "No commands configured yet."
                                    </TableCell>
                                </TableRow>
                            }
                        >
                                <For
                                    each=move || commands.get()
                                    key=|cmd| cmd.id.clone()
                                    children=move |cmd| {
                                        let cmd_id = cmd.id.clone();
                                        let cmd_text = cmd.text.clone();
                                        let cmd_category = cmd.category.clone();
                                        let cmd_enabled = cmd.enabled;
                                        let cmd_action_display = cmd.action_display();
                                        let cmd_clone_for_edit = cmd.clone();

                                        view! {
                                            <TableRow>
                                                <TableCell>
                                                    <code class="bg-gray-100 px-2 py-1 rounded text-sm font-mono">
                                                        {cmd_text}
                                                    </code>
                                                </TableCell>
                                                <TableCell class="text-sm text-gray-600">
                                                    {cmd_action_display}
                                                </TableCell>
                                                <TableCell>{cmd_category}</TableCell>
                                                <TableCell>
                                                    <Badge
                                                        variant=if cmd_enabled { BadgeVariant::Success } else { BadgeVariant::Neutral }
                                                        text=if cmd_enabled { "Enabled" } else { "Disabled" }
                                                    />
                                                </TableCell>
                                                <TableCell>
                                                    <div class="flex gap-2">
                                                        <Button
                                                            variant=ButtonVariant::Secondary
                                                            size=ButtonSize::Small
                                                            on:click=move |_| start_edit(cmd_clone_for_edit.clone())
                                                        >
                                                            "Edit"
                                                        </Button>
                                                        <Button
                                                            variant=ButtonVariant::Danger
                                                            size=ButtonSize::Small
                                                            on:click=move |_| delete_command(cmd_id.clone())
                                                        >
                                                            "Delete"
                                                        </Button>
                                                    </div>
                                                </TableCell>
                                            </TableRow>
                                        }
                                    }
                                />
                        </Show>
                    </DataTable>
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
            </main>
        </div>
    }
}
