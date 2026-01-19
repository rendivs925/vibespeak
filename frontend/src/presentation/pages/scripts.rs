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
        <div class="container">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar active="scripts" />

            <div class="content">
                <h2>"Scripts"</h2>

                <Card title="Script Management">
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                        <p style="margin: 0;">"Execute bash, Python, and JavaScript scripts via voice commands."</p>
                        <button
                            class="btn btn-primary"
                            on:click=move |_| {
                                set_form_name.set("".to_string());
                                set_form_language.set("bash".to_string());
                                set_form_content.set("".to_string());
                                set_form_description.set("".to_string());
                                set_show_create_modal.set(true);
                            }
                        >
                            "Add Script"
                        </button>
                    </div>

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
                                    <th style="padding: 12px; text-align: left; border-bottom: 1px solid #dee2e6; background: #f8f9fa;">"Actions"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || scripts.get()
                                    key=|s| s.id.clone()
                                     children=move |s| {
                                         let s_clone1 = s.clone();
                                         let s_clone2 = s.clone();
                                         view! {
                                             <tr>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{s.name.clone()}</td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">{s.language.clone()}</td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                     {if s.enabled { "Enabled" } else { "Disabled" }}
                                                 </td>
                                                 <td style="padding: 12px; border-bottom: 1px solid #dee2e6;">
                                                     <button
                                                         class="btn btn-sm btn-secondary"
                                                         style="margin-right: 5px;"
                                                         on:click=move |_| start_edit(s_clone1.clone())
                                                     >
                                                         "Edit"
                                                     </button>
                                                     <button
                                                         class="btn btn-sm btn-danger"
                                                         on:click=move |_| delete_script(s_clone2.id.clone())
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

                // Create Script Modal
                <Show when=move || show_create_modal.get()>
                    <div class="modal" style="display: block; background: rgba(0,0,0,0.5); position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 1000;">
                        <div class="modal-dialog" style="max-width: 600px; margin: 50px auto;">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h5 class="modal-title">"Create New Script"</h5>
                                    <button type="button" class="btn-close" on:click=move |_| set_show_create_modal.set(false)></button>
                                </div>
                                <div class="modal-body">
                                    <form>
                                        <div class="mb-3">
                                            <label class="form-label">"Script Name"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_name
                                                on:input=move |e| set_form_name.set(event_target_value(&e))
                                                placeholder="Enter script name"
                                            />
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Language"</label>
                                            <select
                                                class="form-control"
                                                prop:value=form_language
                                                on:change=move |e| set_form_language.set(event_target_value(&e))
                                            >
                                                <option value="bash">"Bash"</option>
                                                <option value="python">"Python"</option>
                                                <option value="javascript">"JavaScript"</option>
                                            </select>
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Description"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_description
                                                on:input=move |e| set_form_description.set(event_target_value(&e))
                                                placeholder="Optional description"
                                            />
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Script Content"</label>
                                            <textarea
                                                class="form-control"
                                                prop:value=form_content
                                                on:input=move |e| set_form_content.set(event_target_value(&e))
                                                placeholder="Enter your script code here"
                                                rows="10"
                                            ></textarea>
                                        </div>
                                    </form>
                                </div>
                                <div class="modal-footer">
                                    <button type="button" class="btn btn-secondary" on:click=move |_| set_show_create_modal.set(false)>"Cancel"</button>
                                    <button type="button" class="btn btn-primary" on:click=move |_| create_script()>"Create Script"</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>

                // Edit Script Modal
                <Show when=move || show_edit_modal.get()>
                    <div class="modal" style="display: block; background: rgba(0,0,0,0.5); position: fixed; top: 0; left: 0; width: 100%; height: 100%; z-index: 1000;">
                        <div class="modal-dialog" style="max-width: 600px; margin: 50px auto;">
                            <div class="modal-content">
                                <div class="modal-header">
                                    <h5 class="modal-title">"Edit Script"</h5>
                                    <button type="button" class="btn-close" on:click=move |_| set_show_edit_modal.set(false)></button>
                                </div>
                                <div class="modal-body">
                                    <form>
                                        <div class="mb-3">
                                            <label class="form-label">"Script Name"</label>
                                            <input
                                                type="text"
                                                class="form-control"
                                                prop:value=form_name
                                                on:input=move |e| set_form_name.set(event_target_value(&e))
                                                placeholder="Enter script name"
                                            />
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Language"</label>
                                            <select
                                                class="form-control"
                                                prop:value=form_language
                                                on:change=move |e| set_form_language.set(event_target_value(&e))
                                            >
                                                <option value="bash">"Bash"</option>
                                                <option value="python">"Python"</option>
                                                <option value="javascript">"JavaScript"</option>
                                            </select>
                                        </div>
                                        <div class="mb-3">
                                            <label class="form-label">"Script Content"</label>
                                            <textarea
                                                class="form-control"
                                                prop:value=form_content
                                                on:input=move |e| set_form_content.set(event_target_value(&e))
                                                placeholder="Enter your script code here"
                                                rows="10"
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
                                            if let Some(s) = editing_script.get() {
                                                update_script(s.id);
                                            }
                                        }
                                    >
                                        "Update Script"
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
