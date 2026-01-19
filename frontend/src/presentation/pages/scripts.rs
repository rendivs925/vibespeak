//! Scripts page component

use crate::domain::entities::Script;
use crate::infrastructure::api_client as api;
use crate::presentation::components::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Card, DataTable, FormField, Header,
    Input, InputType, Modal, NavBar, Select, TableCell, TableRow, Textarea,
};
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn Scripts() -> impl IntoView {
    let (status, set_status) = create_signal("Loading...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (scripts, set_scripts) = create_signal::<Vec<Script>>(vec![]);

    // Modal states
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (editing_script, set_editing_script) = create_signal::<Option<Script>>(None);

    // Form fields
    let (form_name, set_form_name) = create_signal("".to_string());
    let (form_language, set_form_language) = create_signal("bash".to_string());
    let (form_content, set_form_content) = create_signal("".to_string());
    let (form_description, set_form_description) = create_signal("".to_string());

    let load_scripts = move || {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_status.set("Loading scripts...".to_string());
            set_status_type.set("info".to_string());
            match api::ApiClient::new_default().list_scripts().await {
                Ok(response) => {
                    let count = response.scripts.len();
                    set_scripts.set(response.scripts);
                    set_status.set(format!("{} scripts loaded", count));
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to load: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let create_script = move || {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let request = api::CreateScriptRequest {
                name: form_name.get(),
                language: form_language.get(),
                content: form_content.get(),
                description: Some(form_description.get()),
            };

            match api::ApiClient::new_default().create_script(&request).await {
                Ok(_) => {
                    set_show_create_modal.set(false);
                    set_form_name.set("".to_string());
                    set_form_language.set("bash".to_string());
                    set_form_content.set("".to_string());
                    set_form_description.set("".to_string());
                    load_scripts();
                    set_status.set("Script created successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to create script: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let update_script = move |id: String| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let request = api::UpdateScriptRequest {
                name: Some(form_name.get()),
                content: Some(form_content.get()),
                enabled: None,
            };

            match api::ApiClient::new_default()
                .update_script(&id, &request)
                .await
            {
                Ok(_) => {
                    set_show_edit_modal.set(false);
                    set_editing_script.set(None);
                    load_scripts();
                    set_status.set("Script updated successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to update script: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let delete_script = move |id: String| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            if !web_sys::window()
                .unwrap()
                .confirm_with_message("Are you sure you want to delete this script?")
                .unwrap_or(false)
            {
                return;
            }

            match api::ApiClient::new_default().delete_script(&id).await {
                Ok(_) => {
                    load_scripts();
                    set_status.set("Script deleted successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to delete script: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let start_edit = move |s: Script| {
        set_editing_script.set(Some(s.clone()));
        set_form_name.set(s.name);
        set_form_language.set(s.language);
        set_form_content.set(s.content);
        set_show_edit_modal.set(true);
    };

    // Load scripts on mount
    create_effect(move |_| {
        load_scripts();
    });

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50/30">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice" />

            <NavBar _active="scripts" />

                <main class="flex-1 px-4 sm:px-6 lg:px-8 py-4 sm:py-6 lg:py-10 overflow-y-auto">
                <div class="max-w-6xl mx-auto">
                    {/* Page Header */}
                    <div class="mb-10">
                        <h1 class="text-3xl font-semibold text-gray-900 tracking-tight mb-3">
                            "Scripts"
                        </h1>
                        <p class="text-base text-gray-600 leading-relaxed max-w-2xl">
                            "Execute custom scripts and automation routines via voice commands."
                        </p>
                    </div>

                    {/* Content Grid */}
                    <div class="space-y-6">
                        <Card title="Script Management">
                    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
                        <p class="text-gray-600 mb-0">"Execute bash, Python, and JavaScript scripts via voice commands."</p>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Medium
                            on:click=move |_| {
                                set_form_name.set("".to_string());
                                set_form_language.set("bash".to_string());
                                set_form_content.set("".to_string());
                                set_form_description.set("".to_string());
                                set_show_create_modal.set(true);
                            }
                        >
                            "Add Script"
                        </Button>
                    </div>

                    <DataTable headers=vec!["Name".to_string(), "Language".to_string(), "Status".to_string(), "Actions".to_string()]>
                            <Show
                                when=move || !scripts.get().is_empty()
                                fallback=|| view! {
                                    <TableRow>
                                        <TableCell class="text-center text-gray-500 py-8" attr:colspan="4">
                                            "No scripts configured yet."
                                        </TableCell>
                                    </TableRow>
                                }
                            >
                                <For
                                    each=move || scripts.get()
                                    key=|s| s.id.clone()
                                    children=move |s| {
                                        let s_clone1 = s.clone();
                                        let s_clone2 = s.clone();
                                        view! {
                                            <TableRow>
                                                <TableCell class="font-medium text-gray-900">{s.name.clone()}</TableCell>
                                                <TableCell class="text-gray-600">{s.language.clone()}</TableCell>
                                                <TableCell>
                                                    <Badge
                                                        variant=if s.enabled { BadgeVariant::Success } else { BadgeVariant::Neutral }
                                                        text=if s.enabled { "Enabled" } else { "Disabled" }
                                                    />
                                                </TableCell>
                                                <TableCell>
                                                    <div class="flex gap-2">
                                                        <Button
                                                            variant=ButtonVariant::Secondary
                                                            size=ButtonSize::Small
                                                            on:click=move |_| start_edit(s_clone1.clone())
                                                        >
                                                            "Edit"
                                                        </Button>
                                                        <Button
                                                            variant=ButtonVariant::Danger
                                                            size=ButtonSize::Small
                                                            on:click=move |_| delete_script(s_clone2.id.clone())
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

                // Create Script Modal
                <Modal
                    is_open=show_create_modal
                    on_close=Callback::new(move |_: ()| set_show_create_modal.set(false))
                    title="Create New Script"
                    size="lg".to_string()
                >
                    <form class="space-y-6">
                        <FormField label="Script Name">
                            <Input
                                input_type=InputType::Text
                                value=form_name.get()
                                on:input=move |e| set_form_name.set(event_target_value(&e))
                                placeholder="Enter script name".to_string()
                            />
                        </FormField>
                        <FormField label="Language">
                            <Select
                                value=form_language.get()
                                on:change=move |e| set_form_language.set(event_target_value(&e))
                            >
                                <option value="bash">"Bash"</option>
                                <option value="python">"Python"</option>
                                <option value="javascript">"JavaScript"</option>
                            </Select>
                        </FormField>
                        <FormField label="Script Content">
                            <Textarea
                                value=form_content.get()
                                on:input=move |e| set_form_content.set(event_target_value(&e))
                                placeholder="Enter your script code here".to_string()
                                rows=10
                            />
                        </FormField>
                        <FormField label="Language">
                            <Select
                                value=form_language.get()
                                on:change=move |e| set_form_language.set(event_target_value(&e))
                            >
                                <option value="bash">"Bash"</option>
                                <option value="python">"Python"</option>
                                <option value="javascript">"JavaScript"</option>
                            </Select>
                        </FormField>
                        <FormField label="Description" help_text="Optional description of what this script does">
                            <Input
                                input_type=InputType::Text
                                value=form_description.get()
                                on:input=move |e| set_form_description.set(event_target_value(&e))
                                placeholder="Describe what this script does".to_string()
                            />
                        </FormField>
                        <FormField label="Script Content">
                            <Textarea
                                value=form_content.get()
                                on:input=move |e| set_form_content.set(event_target_value(&e))
                                placeholder="Enter your script code here".to_string()
                                rows=10
                            />
                        </FormField>
                        <FormField label="Language">
                            <Select
                                on:change=move |e| set_form_language.set(event_target_value(&e))
                            >
                                <option value="bash" selected=move || form_language.get() == "bash">"Bash"</option>
                                <option value="python" selected=move || form_language.get() == "python">"Python"</option>
                                <option value="javascript" selected=move || form_language.get() == "javascript">"JavaScript"</option>
                            </Select>
                        </FormField>
                        <FormField label="Description" help_text="Optional description of what this script does">
                            <Input
                                input_type=InputType::Text
                                value=form_description.get()
                                on:input=move |e| set_form_description.set(event_target_value(&e))
                                placeholder="Describe what this script does"
                            />
                        </FormField>
                        <FormField label="Script Content">
                            <Textarea
                                value=form_content.get()
                                on:input=move |e| set_form_content.set(event_target_value(&e))
                                placeholder="Enter your script code here"
                                rows=10
                            />
                        </FormField>
                    </form>
                    <div class="flex justify-end gap-3 mt-6">
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Medium
                            on:click=move |_| set_show_create_modal.set(false)
                        >
                            "Cancel"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Medium
                            on:click=move |_| create_script()
                        >
                            "Create Script"
                        </Button>
                    </div>
                </Modal>

                // Edit Script Modal
                <Modal
                    is_open=show_edit_modal
                    on_close=Callback::new(move |_: ()| set_show_edit_modal.set(false))
                    title="Edit Script"
                    size="lg".to_string()
                >
                    <form class="space-y-6">
                        <FormField label="Script Name">
                            <Input
                                input_type=InputType::Text
                                value=form_name.get()
                                on:input=move |e| set_form_name.set(event_target_value(&e))
                                placeholder="Enter script name".to_string()
                            />
                        </FormField>
                        <FormField label="Language">
                            <Select
                                on:change=move |e| set_form_language.set(event_target_value(&e))
                            >
                                <option value="bash" selected=move || form_language.get() == "bash">"Bash"</option>
                                <option value="python" selected=move || form_language.get() == "python">"Python"</option>
                                <option value="javascript" selected=move || form_language.get() == "javascript">"JavaScript"</option>
                            </Select>
                        </FormField>
                        <FormField label="Script Content">
                            <Textarea
                                value=form_content.get()
                                on:input=move |e| set_form_content.set(event_target_value(&e))
                                placeholder="Enter your script code here".to_string()
                                rows=10
                            />
                        </FormField>
                    </form>
                    <div class="flex justify-end gap-3 mt-6">
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Medium
                            on:click=move |_| set_show_edit_modal.set(false)
                        >
                            "Cancel"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Medium
                            on:click=move |_| {
                                if let Some(s) = editing_script.get() {
                                    update_script(s.id);
                                }
                            }
                        >
                            "Update Script"
                        </Button>
                    </div>
                        </Modal>
                    </div>
                </div>
            </main>
        </div>
    }
}
