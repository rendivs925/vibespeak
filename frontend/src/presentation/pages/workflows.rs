//! Workflows page component

use crate::domain::entities::Workflow;
use crate::infrastructure::api_client as api;
use crate::presentation::components::{Card, Header, NavBar, StatusBadge};
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
        <div class="container">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar active="workflows" />

            <div class="content">
                <h2>"Workflows"</h2>

                <Card title="Automation Workflows">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                        <p style="margin: 0;">"Build complex automation sequences triggered by voice commands."</p>
                        <button
                            class="btn btn-primary"
                            on:click=move |_| {
                                set_form_name.set("".to_string());
                                set_form_description.set("".to_string());
                                set_show_create_modal.set(true);
                            }
                        >
                            "Add Workflow"
                        </button>
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
                        <table style="width: 100%; border-collapse: collapse; margin-top: 20px;">
                            <thead>
                                <tr>
                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Name"</th>
                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Description"</th>
                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Steps"</th>
                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Status"</th>
                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || workflows.get()
                                    key=|wf| wf.id.clone()
                                     children=move |wf| {
                                         let wf_clone1 = wf.clone();
                                         let wf_clone2 = wf.clone();
                                         view! {
                                             <tr>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{wf.name.clone()}</td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{wf.description.clone()}</td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{wf.steps.len()}" steps"</td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                     {if wf.enabled { "Enabled" } else { "Disabled" }}
                                                 </td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                     <button
                                                         class="btn btn-sm btn-secondary"
                                                         style="margin-right: 5px;"
                                                         on:click=move |_| start_edit(wf_clone1.clone())
                                                     >
                                                         "Edit"
                                                     </button>
                                                     <button
                                                         class="btn btn-sm btn-danger"
                                                         on:click=move |_| delete_workflow(wf_clone2.id.clone())
                                                     >
                                                         "Delete"
                                                     </button>
                                                 </td>
                                             </tr>
                                         }
                                     }
                                />
                            </tbody>
                        </table>
                    </Show>
                </Card>

                // Create Workflow Modal
                <Show when=move || show_create_modal.get()>
                    <div class="modal" style="display: block; background: rgba(0,0,0,0.5); position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 1000;">
                        <div class="modal-dialog" style="max-width: 500px; margin: 50px auto;">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h5 class="modal-title">"Create New Workflow"</h5>
                                    <button type="button" class="btn-close" on:click=move |_| set_show_create_modal.set(false)></button>
                                </div>
                                <div class="modal-body">
                                    <form>
                                        <div class="mb-3">
                                            <label class="form-label">"Workflow Name"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_name
                                                on:input=move |e| set_form_name.set(event_target_value(&e))
                                                placeholder="Enter workflow name"
                                            />
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Description"</label>
                                            <textarea
                                                class="form-control"
                                                prop:value=form_description
                                                on:input=move |e| set_form_description.set(event_target_value(&e))
                                                placeholder="Describe what this workflow does"
                                                rows="3"
                                            ></textarea>
                                        </div>
                                    </form>
                                </div>
                                <div class="modal-footer">
                                    <button type="button" class="btn btn-secondary" on:click=move |_| set_show_create_modal.set(false)>"Cancel"</button>
                                    <button type="button" class="btn btn-primary" on:click=move |_| create_workflow()>"Create Workflow"</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>

                // Edit Workflow Modal
                <Show when=move || show_edit_modal.get()>
                    <div class="modal" style="display: block; background: rgba(0,0,0,0.5); position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 1000;">
                        <div class="modal-dialog" style="max-width: 500px; margin: 50px auto;">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h5 class="modal-title">"Edit Workflow"</h5>
                                    <button type="button" class="btn-close" on:click=move |_| set_show_edit_modal.set(false)></button>
                                </div>
                                <div class="modal-body">
                                    <form>
                                        <div class="mb-3">
                                            <label class="form-label">"Workflow Name"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_name
                                                on:input=move |e| set_form_name.set(event_target_value(&e))
                                                placeholder="Enter workflow name"
                                            />
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Description"</label>
                                            <textarea
                                                class="form-control"
                                                prop:value=form_description
                                                on:input=move |e| set_form_description.set(event_target_value(&e))
                                                placeholder="Describe what this workflow does"
                                                rows="3"
                                            ></textarea>
                                        </div>
                                    </form>
                                </div>
                                <div class="modal-footer">
                                    <button type="button" class="btn btn-secondary" on:click=move |_| set_show_edit_modal.set(false)>"Cancel"</button>
                                    <button
                                        type="button"
                                        class="btn btn-primary"
                                        on:click=move |_| {
                                            if let Some(wf) = editing_workflow.get() {
                                                update_workflow(wf.id);
                                            }
                                        }
                                    >
                                        "Update Workflow"
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
