//! Voice Commands page component - Ultra Premium Design

use crate::domain::entities::Command;
use crate::infrastructure::api_client as api;
use crate::presentation::components::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, DataTable, Header,
    Icon, IconType, Modal, ModalFooter, NavBar, StatusBadge, TableCell, TableRow,
};
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn VoiceCommands() -> impl IntoView {
    let (status, set_status) = create_signal("Loading...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (commands, set_commands) = create_signal::<Vec<Command>>(vec![]);
    let (filter_text, set_filter_text) = create_signal(String::new());

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

    // Filtered commands
    let filtered_commands = move || {
        let filter = filter_text.get().to_lowercase();
        if filter.is_empty() {
            commands.get()
        } else {
            commands
                .get()
                .into_iter()
                .filter(|cmd| {
                    cmd.text.to_lowercase().contains(&filter)
                        || cmd.category.to_lowercase().contains(&filter)
                })
                .collect()
        }
    };

    create_effect(move |_| {
        load_commands();
    });

    // Create closures for modal close callbacks
    let close_create_modal = Callback::new(move |_: ()| set_show_create_modal.set(false));
    let close_edit_modal = Callback::new(move |_: ()| set_show_edit_modal.set(false));

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50/30">
            <Header title="Vibespeak" subtitle="Voice Automation System">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar _active="commands" />

            <main class="flex-1 px-4 sm:px-8 py-6 sm:py-10 overflow-y-auto">
                <div class="max-w-6xl mx-auto">
                    // Page Header
                    <div class="mb-8 sm:mb-10">
                        <p class="text-xs font-semibold tracking-wide uppercase text-indigo-600 mb-2">
                            "Voice Commands"
                        </p>
                        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                            <div>
                                <h1 class="text-2xl sm:text-3xl font-semibold text-gray-900 tracking-tight mb-2">
                                    "Voice Commands"
                                </h1>
                                <p class="text-sm sm:text-base text-gray-600 leading-relaxed max-w-2xl">
                                    "Create and manage voice commands to control your applications hands-free."
                                </p>
                            </div>
                            <Button
                                variant=ButtonVariant::Primary
                                on:click=move |_| {
                                    set_form_text.set("".to_string());
                                    set_form_category.set("general".to_string());
                                    set_form_action_type.set("ShellCommand".to_string());
                                    set_form_action_value.set("".to_string());
                                    set_show_create_modal.set(true);
                                }
                            >
                                <Icon icon_type=IconType::Plus class="w-4 h-4 mr-2" />
                                "Add Command"
                            </Button>
                        </div>
                    </div>

                    // Stats Cards
                    <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-6">
                        <div class="bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 p-4">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-indigo-600 flex items-center justify-center">
                                    <Icon icon_type=IconType::Commands class="w-5 h-5 text-white" />
                                </div>
                                <div>
                                    <p class="text-2xl font-semibold text-gray-900">{move || commands.get().len()}</p>
                                    <p class="text-xs text-slate-500">"Total Commands"</p>
                                </div>
                            </div>
                        </div>
                        <div class="bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 p-4">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500 to-emerald-600 flex items-center justify-center">
                                    <Icon icon_type=IconType::Check class="w-5 h-5 text-white" />
                                </div>
                                <div>
                                    <p class="text-2xl font-semibold text-gray-900">
                                        {move || commands.get().iter().filter(|c| c.enabled).count()}
                                    </p>
                                    <p class="text-xs text-slate-500">"Enabled"</p>
                                </div>
                            </div>
                        </div>
                        <div class="bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 p-4">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-amber-500 to-amber-600 flex items-center justify-center">
                                    <Icon icon_type=IconType::Workflows class="w-5 h-5 text-white" />
                                </div>
                                <div>
                                    <p class="text-2xl font-semibold text-gray-900">
                                        {move || {
                                            let cmds = commands.get();
                                            let categories: std::collections::HashSet<_> = cmds.iter().map(|c| &c.category).collect();
                                            categories.len()
                                        }}
                                    </p>
                                    <p class="text-xs text-slate-500">"Categories"</p>
                                </div>
                            </div>
                        </div>
                        <div class="bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 p-4">
                            <div class="flex items-center gap-3">
                                <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-rose-500 to-rose-600 flex items-center justify-center">
                                    <Icon icon_type=IconType::Commands class="w-5 h-5 text-white" />
                                </div>
                                <div>
                                    <p class="text-2xl font-semibold text-gray-900">
                                        {move || commands.get().iter().filter(|c| c.action.get("ShellCommand").is_some()).count()}
                                    </p>
                                    <p class="text-xs text-slate-500">"Shell Commands"</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Commands Table Card
                    <section class="relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 overflow-hidden">
                        // Card Header with Search
                        <div class="px-6 sm:px-7 py-5 border-b border-slate-100 bg-gradient-to-r from-white to-slate-50/30">
                            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                                <div class="flex items-center gap-3">
                                    <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-indigo-600 flex items-center justify-center shadow-md shadow-indigo-200/50">
                                        <Icon icon_type=IconType::Commands class="w-5 h-5 text-white" />
                                    </div>
                                    <div>
                                        <h3 class="text-lg font-semibold text-gray-900 tracking-tight">"Command Library"</h3>
                                        <p class="text-xs text-slate-500">"Say the voice text to trigger the action"</p>
                                    </div>
                                </div>
                                <div class="relative">
                                    <Icon icon_type=IconType::Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
                                    <input
                                        type="text"
                                        class="pl-10 pr-4 py-2 text-sm text-gray-900 bg-slate-50 border border-slate-200/80 rounded-xl placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 focus:bg-white transition-all duration-200 w-full sm:w-64"
                                        placeholder="Search commands..."
                                        prop:value=move || filter_text.get()
                                        on:input=move |ev| set_filter_text.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                        </div>

                        // Table
                        <div class="overflow-x-auto">
                            <DataTable headers=vec!["Voice Text".to_string(), "Action".to_string(), "Category".to_string(), "Status".to_string(), "".to_string()]>
                                <Show
                                    when=move || !filtered_commands().is_empty()
                                    fallback=|| view! {
                                        <TableRow>
                                            <TableCell class="text-center py-12" attr:colspan="5">
                                                <div class="flex flex-col items-center">
                                                    <div class="w-16 h-16 mb-4 rounded-full bg-slate-100 flex items-center justify-center">
                                                        <Icon icon_type=IconType::Commands class="w-8 h-8 text-slate-400" />
                                                    </div>
                                                    <p class="text-sm font-medium text-slate-600">"No commands found"</p>
                                                    <p class="text-xs text-slate-400 mt-1">"Try adjusting your search or create a new command"</p>
                                                </div>
                                            </TableCell>
                                        </TableRow>
                                    }
                                >
                                    <For
                                        each=move || filtered_commands()
                                        key=|cmd| cmd.id.clone()
                                        children=move |cmd| {
                                            let cmd_id = cmd.id.clone();
                                            let cmd_id_delete = cmd.id.clone();
                                            let cmd_text = cmd.text.clone();
                                            let cmd_category = cmd.category.clone();
                                            let cmd_enabled = cmd.enabled;
                                            let cmd_action_display = cmd.action_display();
                                            let cmd_clone_for_edit = cmd.clone();

                                            view! {
                                                <TableRow class="group hover:bg-slate-50/50 transition-colors duration-200">
                                                    <TableCell>
                                                        <div class="flex items-center gap-3">
                                                            <div class="w-8 h-8 rounded-lg bg-indigo-50 flex items-center justify-center">
                                                                <Icon icon_type=IconType::Microphone class="w-4 h-4 text-indigo-600" />
                                                            </div>
                                                            <code class="text-sm font-medium text-gray-900 bg-slate-100 px-2 py-1 rounded-lg">
                                                                {cmd_text}
                                                            </code>
                                                        </div>
                                                    </TableCell>
                                                    <TableCell>
                                                        <p class="text-sm text-gray-600 font-mono truncate max-w-xs">{cmd_action_display}</p>
                                                    </TableCell>
                                                    <TableCell>
                                                        <Badge
                                                            variant=BadgeVariant::Neutral
                                                            text=cmd_category
                                                        />
                                                    </TableCell>
                                                    <TableCell>
                                                        <Badge
                                                            variant=if cmd_enabled { BadgeVariant::Success } else { BadgeVariant::Neutral }
                                                            text=if cmd_enabled { "Enabled" } else { "Disabled" }
                                                        />
                                                    </TableCell>
                                                    <TableCell>
                                                        <div class="flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                                                            <button
                                                                class="p-1.5 rounded-lg text-slate-400 hover:text-indigo-600 hover:bg-indigo-50 transition-all duration-200"
                                                                on:click=move |_| start_edit(cmd_clone_for_edit.clone())
                                                            >
                                                                <Icon icon_type=IconType::Edit class="w-4 h-4" />
                                                            </button>
                                                            <button
                                                                class="p-1.5 rounded-lg text-slate-400 hover:text-red-600 hover:bg-red-50 transition-all duration-200"
                                                                on:click=move |_| delete_command(cmd_id_delete.clone())
                                                            >
                                                                <Icon icon_type=IconType::X class="w-4 h-4" />
                                                            </button>
                                                        </div>
                                                    </TableCell>
                                                </TableRow>
                                            }
                                        }
                                    />
                                </Show>
                            </DataTable>
                        </div>
                    </section>
                </div>
            </main>

            // Create Command Modal
            <Modal
                is_open=show_create_modal.into()
                on_close=close_create_modal
                title="Create New Command".to_string()
                subtitle=Some("Add a new voice command to your library".to_string())
                icon=Some(IconType::Plus)
                size="lg".to_string()
            >
                <div class="space-y-5">
                    <div class="space-y-2">
                        <label class="block text-sm font-medium text-gray-700">"Voice Trigger"</label>
                        <input
                            type="text"
                            class="w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200"
                            placeholder="What you say to trigger this command"
                            prop:value=form_text
                            on:input=move |e| set_form_text.set(event_target_value(&e))
                        />
                        <p class="text-xs text-slate-500">"This is the phrase you'll say to activate the command"</p>
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div class="space-y-2">
                            <label class="block text-sm font-medium text-gray-700">"Category"</label>
                            <input
                                type="text"
                                class="w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200"
                                placeholder="e.g., general, browser"
                                prop:value=form_category
                                on:input=move |e| set_form_category.set(event_target_value(&e))
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="block text-sm font-medium text-gray-700">"Action Type"</label>
                            <select
                                class="w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200"
                                prop:value=form_action_type
                                on:change=move |e| set_form_action_type.set(event_target_value(&e))
                            >
                                <option value="ShellCommand">"Shell Command"</option>
                                <option value="Workflow">"Workflow"</option>
                                <option value="Script">"Script"</option>
                            </select>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="block text-sm font-medium text-gray-700">"Action Value"</label>
                        <textarea
                            class="w-full px-4 py-3 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200 resize-none font-mono"
                            rows="3"
                            placeholder=move || match form_action_type.get().as_str() {
                                "ShellCommand" => "e.g., xdotool key ctrl+t",
                                "Workflow" => "Workflow ID",
                                "Script" => "Script ID",
                                _ => "Action value",
                            }
                            prop:value=form_action_value
                            on:input=move |e| set_form_action_value.set(event_target_value(&e))
                        ></textarea>
                    </div>

                    <ModalFooter>
                        <Button variant=ButtonVariant::Ghost on:click=move |_| set_show_create_modal.set(false)>
                            "Cancel"
                        </Button>
                        <Button variant=ButtonVariant::Primary on:click=move |_| create_command()>
                            <Icon icon_type=IconType::Plus class="w-4 h-4 mr-2" />
                            "Create Command"
                        </Button>
                    </ModalFooter>
                </div>
            </Modal>

            // Edit Command Modal
            <Modal
                is_open=show_edit_modal.into()
                on_close=close_edit_modal
                title="Edit Command".to_string()
                subtitle=Some("Update your voice command configuration".to_string())
                icon=Some(IconType::Edit)
                size="lg".to_string()
            >
                <div class="space-y-5">
                    <div class="space-y-2">
                        <label class="block text-sm font-medium text-gray-700">"Voice Trigger"</label>
                        <input
                            type="text"
                            class="w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200"
                            placeholder="What you say to trigger this command"
                            prop:value=form_text
                            on:input=move |e| set_form_text.set(event_target_value(&e))
                        />
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div class="space-y-2">
                            <label class="block text-sm font-medium text-gray-700">"Category"</label>
                            <input
                                type="text"
                                class="w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200"
                                placeholder="e.g., general, browser"
                                prop:value=form_category
                                on:input=move |e| set_form_category.set(event_target_value(&e))
                            />
                        </div>
                        <div class="space-y-2">
                            <label class="block text-sm font-medium text-gray-700">"Action Type"</label>
                            <select
                                class="w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200"
                                prop:value=form_action_type
                                on:change=move |e| set_form_action_type.set(event_target_value(&e))
                            >
                                <option value="ShellCommand">"Shell Command"</option>
                                <option value="Workflow">"Workflow"</option>
                                <option value="Script">"Script"</option>
                            </select>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="block text-sm font-medium text-gray-700">"Action Value"</label>
                        <textarea
                            class="w-full px-4 py-3 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200 resize-none font-mono"
                            rows="3"
                            prop:value=form_action_value
                            on:input=move |e| set_form_action_value.set(event_target_value(&e))
                        ></textarea>
                    </div>

                    <ModalFooter>
                        <Button variant=ButtonVariant::Ghost on:click=move |_| set_show_edit_modal.set(false)>
                            "Cancel"
                        </Button>
                        <Button variant=ButtonVariant::Primary on:click=move |_| {
                            if let Some(cmd) = editing_command.get() {
                                update_command(cmd.id);
                            }
                        }>
                            <Icon icon_type=IconType::Check class="w-4 h-4 mr-2" />
                            "Save Changes"
                        </Button>
                    </ModalFooter>
                </div>
            </Modal>
        </div>
    }
}
