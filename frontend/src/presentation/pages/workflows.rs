//! Workflows page component

use crate::domain::entities::Workflow;
use crate::infrastructure::api_client as api;
use crate::presentation::components::{
    Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Card, DataTable, FormField, Header,
    Input, InputType, Modal, NavBar, StatusBadge, TableCell, TableRow, Textarea,
};
use leptos::*;
use wasm_bindgen_futures;

#[component]
pub fn Workflows() -> impl IntoView {
    let (status, set_status) = create_signal("Loading...".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (workflows, set_workflows) = create_signal::<Vec<Workflow>>(vec![]);

    // Modal states
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (editing_workflow, set_editing_workflow) = create_signal::<Option<Workflow>>(None);

    // Form fields
    let (form_name, set_form_name) = create_signal("".to_string());
    let (form_description, set_form_description) = create_signal("".to_string());

    let load_workflows = move || {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_status.set("Loading workflows...".to_string());
            set_status_type.set("info".to_string());
            match api::ApiClient::new_default().list_workflows().await {
                Ok(response) => {
                    let count = response.workflows.len();
                    set_workflows.set(response.workflows);
                    set_status.set(format!("{} workflows loaded", count));
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to load: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let create_workflow = move || {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let request = api::CreateWorkflowRequest {
                name: form_name.get(),
                description: form_description.get(),
            };

            match api::ApiClient::new_default()
                .create_workflow(&request)
                .await
            {
                Ok(_) => {
                    set_show_create_modal.set(false);
                    set_form_name.set("".to_string());
                    set_form_description.set("".to_string());
                    load_workflows();
                    set_status.set("Workflow created successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to create workflow: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let update_workflow = move |id: String| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            let request = api::UpdateWorkflowRequest {
                name: Some(form_name.get()),
                description: Some(form_description.get()),
                enabled: None,
            };

            match api::ApiClient::new_default()
                .update_workflow(&id, &request)
                .await
            {
                Ok(_) => {
                    set_show_edit_modal.set(false);
                    set_editing_workflow.set(None);
                    load_workflows();
                    set_status.set("Workflow updated successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to update workflow: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let delete_workflow = move |id: String| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            if !web_sys::window()
                .unwrap()
                .confirm_with_message("Are you sure you want to delete this workflow?")
                .unwrap_or(false)
            {
                return;
            }

            match api::ApiClient::new_default().delete_workflow(&id).await {
                Ok(_) => {
                    load_workflows();
                    set_status.set("Workflow deleted successfully".to_string());
                    set_status_type.set("success".to_string());
                }
                Err(e) => {
                    set_status.set(format!("Failed to delete workflow: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let start_edit = move |wf: Workflow| {
        set_editing_workflow.set(Some(wf.clone()));
        set_form_name.set(wf.name);
        set_form_description.set(wf.description);
        set_show_edit_modal.set(true);
    };

    // Load workflows on mount
    create_effect(move |_| {
        load_workflows();
    });

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50/30">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar _active="workflows" />

            <main class="flex-1 px-8 py-10 overflow-y-auto">
                <div class="max-w-6xl mx-auto">
                    {/* Page Header */}
                    <div class="mb-10">
                        <h1 class="text-3xl font-semibold text-gray-900 tracking-tight mb-3">
                            "Workflows"
                        </h1>
                        <p class="text-base text-gray-600 leading-relaxed max-w-2xl">
                            "Create and manage automated sequences of voice commands."
                        </p>
                    </div>

                    {/* Content Grid */}
                    <div class="space-y-6">
                        <Card title="Automation Workflows">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                        <p style="margin: 0;">"Build complex automation sequences triggered by voice commands."</p>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Medium
                            on:click=move |_| {
                                set_form_name.set("".to_string());
                                set_form_description.set("".to_string());
                                set_show_create_modal.set(true);
                            }
                        >
                            "Add Workflow"
                        </Button>
                    </div>

                    <Show
                        when=move || !workflows.get().is_empty()
                        fallback=|| view! {
                            <div style="margin-top: 20px; padding: 20px; background: #f8f9fa; border-radius: 4px; text-align: center;">
                                <p style="color: #6c757d;">"No workflows configured yet."</p>
                                <p style="font-size: 14px; color: #868e96; margin-top: 10px;">
                                    "Workflows allow you to chain multiple commands together."
                                </p>
                            </div>
                        }
                    >
                        <DataTable headers=vec!["Name".to_string(), "Description".to_string(), "Steps".to_string(), "Status".to_string(), "Actions".to_string()]>
                            <For
                                each=move || workflows.get()
                                key=|wf| wf.id.clone()
                                children=move |wf| {
                                    let wf_clone1 = wf.clone();
                                    let wf_clone2 = wf.clone();
                                    view! {
                                        <TableRow>
                                            <TableCell class="font-medium text-gray-900">{wf.name.clone()}</TableCell>
                                            <TableCell class="text-gray-600">{wf.description.clone()}</TableCell>
                                            <TableCell class="text-gray-600">{wf.steps.len()}" steps"</TableCell>
                                            <TableCell>
                                                <Badge
                                                    variant=if wf.enabled { BadgeVariant::Success } else { BadgeVariant::Neutral }
                                                    text=if wf.enabled { "Enabled" } else { "Disabled" }
                                                />
                                            </TableCell>
                                            <TableCell>
                                                <div class="flex gap-2">
                                                    <Button
                                                        variant=ButtonVariant::Secondary
                                                        size=ButtonSize::Small
                                                        on:click=move |_| start_edit(wf_clone1.clone())
                                                    >
                                                        "Edit"
                                                    </Button>
                                                    <Button
                                                        variant=ButtonVariant::Danger
                                                        size=ButtonSize::Small
                                                        on:click=move |_| delete_workflow(wf_clone2.id.clone())
                                                    >
                                                        "Delete"
                                                    </Button>
                                                </div>
                                            </TableCell>
                                        </TableRow>
                                    }
                                }
                            />
                        </DataTable>
                    </Show>
                </Card>

                // Create Workflow Modal
                <Modal
                    is_open=show_create_modal
                    title="Create New Workflow"
                >
                    <form class="space-y-6">
                        <FormField label="Workflow Name">
                            <Input
                                input_type=InputType::Text
                                value=form_name.get()
                                on:input=move |e| set_form_name.set(event_target_value(&e))
                                placeholder="Enter workflow name".to_string()
                            />
                        </FormField>
                        <FormField label="Description" help_text="Describe what this workflow does">
                            <Textarea
                                value=form_description.get()
                                on:input=move |e| set_form_description.set(event_target_value(&e))
                                placeholder="Describe what this workflow does".to_string()
                                rows=3
                            />
                        </FormField>
                        <FormField label="Description" help_text="Describe what this workflow does">
                            <Textarea
                                value=form_description.get()
                                on:input=move |e| set_form_description.set(event_target_value(&e))
                                placeholder="Describe what this workflow does".to_string()
                                rows=3
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
                            on:click=move |_| create_workflow()
                        >
                            "Create Workflow"
                        </Button>
                    </div>
                </Modal>

                // Edit Workflow Modal
                <Modal
                    is_open=show_edit_modal
                    title="Edit Workflow"
                >
                    <form class="space-y-6">
                        <FormField label="Workflow Name">
                            <Input
                                input_type=InputType::Text
                                value=form_name.get()
                                on:input=move |e| set_form_name.set(event_target_value(&e))
                                placeholder="Enter workflow name".to_string()
                            />
                        </FormField>
                        <FormField label="Description" help_text="Describe what this workflow does">
                            <Textarea
                                value=form_description.get()
                                on:input=move |e| set_form_description.set(event_target_value(&e))
                                placeholder="Describe what this workflow does".to_string()
                                rows=3
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
                                if let Some(wf) = editing_workflow.get() {
                                    update_workflow(wf.id);
                                }
                            }
                        >
                            "Update Workflow"
                        </Button>
                    </div>
                        </Modal>
                    </div>
                </div>
            </main>
        </div>
    }
}
