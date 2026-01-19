//! Remote Control page component

use crate::infrastructure::api_client as api;
use crate::presentation::components::{Card, Header, NavBar, StatusBadge};
use leptos::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures;
use std::cell::RefCell;
use std::rc::Rc;

#[component]
pub fn RemoteControl() -> impl IntoView {
    let (status, set_status) = create_signal("Ready".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (dictation_text, set_dictation_text) = create_signal(String::new());
    let (dictation_status, set_dictation_status) = create_signal("Dictation ready".to_string());
    let (is_dictating, set_is_dictating) = create_signal(false);
    let (commands_history, set_commands_history) = create_signal::<Vec<String>>(vec![]);

    // Store reference to SpeechRecognition instance
    let recognition_ref: Rc<RefCell<Option<web_sys::SpeechRecognition>>> = Rc::new(RefCell::new(None));

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

    let clear_dictation = move |_| {
        set_dictation_text.set(String::new());
        set_dictation_status.set("Dictation ready".to_string());
    };

    let test_keyboard = move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_dictation_status.set("Testing keyboard simulation...".to_string());
            match api::ApiClient::new_default().test_keyboard().await {
                Ok(_) => {
                    set_dictation_status
                        .set("Keyboard test completed - check if you saw text appear!".to_string());
                }
                Err(e) => {
                    set_dictation_status.set(format!("Keyboard test failed: {}", e));
                }
            }
        });
        #[cfg(not(target_arch = "wasm32"))]
        leptos::create_effect(move |_| {
            leptos::spawn_local(async move {
                set_dictation_status
                    .set("Keyboard test not available in non-WASM environment".to_string());
            });
        });
    };

    // Real-time dictation with Web Speech API
    let recognition_ref_start = recognition_ref.clone();
    let start_dictation = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;

            // Create SpeechRecognition instance
            let recognition = web_sys::SpeechRecognition::new();

            match recognition {
                Ok(recognition) => {
                    // Configure for continuous real-time dictation
                    recognition.set_continuous(true);
                    recognition.set_interim_results(true);
                    recognition.set_lang("en-US");

                    // Clone signals for closures
                    let set_dictation_status_result = set_dictation_status.clone();
                    let set_dictation_text_result = set_dictation_text.clone();
                    let set_is_dictating_result = set_is_dictating.clone();

                    // Handle speech recognition results
                    let onresult = Closure::wrap(Box::new(move |event: web_sys::SpeechRecognitionEvent| {
                        let results = event.results();
                        if let Some(results) = results {
                            // Get the latest result
                            let result_index = event.result_index();
                            if let Some(result) = results.get(result_index) {
                                if let Some(alternative) = result.get(0) {
                                    let transcript = alternative.transcript();
                                    let is_final = result.is_final();

                                    // Update display
                                    set_dictation_text_result.set(transcript.clone());

                                    if is_final && !transcript.trim().is_empty() {
                                        // Final result - send to backend to type
                                        let text = transcript.clone();
                                        set_dictation_status_result.set(format!("⌨️ Typing: \"{}\"", text));

                                        wasm_bindgen_futures::spawn_local(async move {
                                            match api::ApiClient::new_default().type_dictation(&text).await {
                                                Ok(response) => {
                                                    if response.success {
                                                        web_sys::console::log_1(&format!("Typed {} chars", response.characters_typed).into());
                                                    } else {
                                                        web_sys::console::error_1(&format!("Type failed: {:?}", response.error).into());
                                                    }
                                                }
                                                Err(e) => {
                                                    web_sys::console::error_1(&format!("API error: {}", e).into());
                                                }
                                            }
                                        });

                                        // Clear for next phrase
                                        set_dictation_text_result.set(String::new());
                                    } else if !is_final {
                                        set_dictation_status_result.set(format!("🎤 Listening: \"{}\"...", transcript));
                                    }
                                }
                            }
                        }
                    }) as Box<dyn FnMut(_)>);

                    recognition.set_onresult(Some(onresult.as_ref().unchecked_ref()));
                    onresult.forget();

                    // Handle errors
                    let set_dictation_status_error = set_dictation_status.clone();
                    let set_is_dictating_error = set_is_dictating.clone();
                    let onerror = Closure::wrap(Box::new(move |event: web_sys::Event| {
                        set_dictation_status_error.set("❌ Speech recognition error - try again".to_string());
                        set_is_dictating_error.set(false);
                        web_sys::console::error_1(&format!("Speech error: {:?}", event).into());
                    }) as Box<dyn FnMut(_)>);

                    recognition.set_onerror(Some(onerror.as_ref().unchecked_ref()));
                    onerror.forget();

                    // Handle end (auto-restart for continuous mode)
                    let recognition_clone = recognition.clone();
                    let set_dictation_status_end = set_dictation_status.clone();
                    let is_dictating_end = is_dictating.clone();
                    let onend = Closure::wrap(Box::new(move |_: web_sys::Event| {
                        // Auto-restart if still in dictation mode
                        if is_dictating_end.get_untracked() {
                            set_dictation_status_end.set("🎤 Restarting...".to_string());
                            let _ = recognition_clone.start();
                        }
                    }) as Box<dyn FnMut(_)>);

                    recognition.set_onend(Some(onend.as_ref().unchecked_ref()));
                    onend.forget();

                    // Start recognition
                    match recognition.start() {
                        Ok(_) => {
                            set_dictation_status.set("🎤 Listening... speak now (auto-typing enabled)".to_string());
                            set_is_dictating.set(true);
                            *recognition_ref_start.borrow_mut() = Some(recognition);
                        }
                        Err(e) => {
                            set_dictation_status.set(format!("Failed to start: {:?}", e));
                        }
                    }
                }
                Err(e) => {
                    set_dictation_status.set(format!("Speech recognition not supported: {:?}", e));
                }
            }
        }
    };

    let recognition_ref_stop = recognition_ref.clone();
    let stop_dictation = move |_| {
        set_is_dictating.set(false);
        if let Some(recognition) = recognition_ref_stop.borrow_mut().take() {
            let _ = recognition.stop();
        }
        set_dictation_status.set("⏹️ Dictation stopped".to_string());
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
                    <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                        <div style="display: flex; gap: 5px;">
                        <button class="btn btn-success" on:click=start_dictation>
                            "🎤 Dictation Mode"
                        </button>
                            <button class="btn btn-danger" on:click=stop_dictation>
                                "⏹️ Stop"
                            </button>
                        </div>
                        <div style="display: flex; gap: 5px;">
                            <button class="btn btn-secondary" on:click=test_keyboard>
                                "Test Keyboard"
                            </button>
                            <button class="btn btn-outline" on:click=clear_dictation>
                                "Clear"
                            </button>
                        </div>
                    </div>
                    <div style="margin-top: 15px; padding: 10px; background: #e9ecef; border-radius: 4px; font-size: 14px;">
                        <strong>"How to use dictation:"</strong>
                        <ol style="margin: 5px 0; padding-left: 20px;">
                            <li><strong>"Switch to your target application first"</strong>" (Gmail, VS Code, browser, etc.)"</li>
                            <li>"Type or paste text in the field above"</li>
                            <li>"Click \"Type Text\" to send keystrokes automatically"</li>
                        </ol>
                         <div style="margin-top: 10px; padding: 8px; background: #d1ecf1; border: 1px solid #bee5eb; border-radius: 3px;">
                            <strong>"Real-Time Dictation:"</strong>" As you speak, text is automatically typed into "<strong>"any application"</strong>" that has focus - true hands-free voice typing!"
                        </div>
                        <div style="margin-top: 10px; padding: 8px; background: #fff3cd; border: 1px solid #ffeaa7; border-radius: 3px;">
                            <strong>"System Setup:"</strong>
                            <ul style="margin: 5px 0; padding-left: 20px; font-size: 13px;">
                                <li>"Test keyboard simulation with the 'Test Keyboard' button first"</li>
                                <li>"uinput (preferred): Requires root or udev rules for /dev/uinput"</li>
                                <li>"xdotool (fallback): Requires X11 display access (DISPLAY=:0)"</li>
                                <li>"Switch to target app before clicking 'Type Text'"</li>
                            </ul>
                        </div>
                    </div>
                </Card>
            </div>
        </div>
    }
}
