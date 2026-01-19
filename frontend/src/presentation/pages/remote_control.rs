//! Remote Control page component

use crate::infrastructure::api_client as api;
use crate::presentation::components::{Card, Header, NavBar, StatusBadge};
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures;

#[component]
pub fn RemoteControl() -> impl IntoView {
    let (status, set_status) = create_signal("Ready".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (dictation_text, set_dictation_text) = create_signal(String::new());
    let (dictation_status, set_dictation_status) = create_signal("Dictation ready".to_string());
    let (is_dictating, set_is_dictating) = create_signal(false);
    let (commands_history, set_commands_history) = create_signal::<Vec<String>>(vec![]);

    // Screen sharing state
    let (screen_status, set_screen_status) =
        create_signal("Screen sharing not started".to_string());
    let (is_screen_sharing, set_is_screen_sharing) = create_signal(false);
    let screen_stream_ref: Rc<RefCell<Option<web_sys::MediaStream>>> = Rc::new(RefCell::new(None));

    // Store reference to SpeechRecognition instance
    let recognition_ref: Rc<RefCell<Option<web_sys::SpeechRecognition>>> =
        Rc::new(RefCell::new(None));

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
                    let onresult =
                        Closure::wrap(Box::new(move |event: web_sys::SpeechRecognitionEvent| {
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
                                            set_dictation_status_result
                                                .set(format!("⌨️ Typing: \"{}\"", text));

                                            wasm_bindgen_futures::spawn_local(async move {
                                                match api::ApiClient::new_default()
                                                    .type_dictation(&text)
                                                    .await
                                                {
                                                    Ok(response) => {
                                                        if response.success {
                                                            web_sys::console::log_1(
                                                                &format!(
                                                                    "Typed {} chars",
                                                                    response.characters_typed
                                                                )
                                                                .into(),
                                                            );
                                                        } else {
                                                            web_sys::console::error_1(
                                                                &format!(
                                                                    "Type failed: {:?}",
                                                                    response.error
                                                                )
                                                                .into(),
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        web_sys::console::error_1(
                                                            &format!("API error: {}", e).into(),
                                                        );
                                                    }
                                                }
                                            });

                                            // Clear for next phrase
                                            set_dictation_text_result.set(String::new());
                                        } else if !is_final {
                                            set_dictation_status_result.set(format!(
                                                "🎤 Listening: \"{}\"...",
                                                transcript
                                            ));
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
                        set_dictation_status_error
                            .set("❌ Speech recognition error - try again".to_string());
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
                            set_dictation_status
                                .set("🎤 Listening... speak now (auto-typing enabled)".to_string());
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

    // Screen sharing functions using JavaScript interop
    let screen_stream_ref_start = screen_stream_ref.clone();
    let start_screen_share = move |_| {
        let screen_stream_ref_inner = screen_stream_ref_start.clone();
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;

            set_screen_status.set("Requesting screen access...".to_string());

            wasm_bindgen_futures::spawn_local(async move {
                let window = web_sys::window().unwrap();
                let navigator = window.navigator();

                // Get media devices
                let media_devices = match navigator.media_devices() {
                    Ok(md) => md,
                    Err(_) => {
                        set_screen_status.set("Media devices not available".to_string());
                        return;
                    }
                };

                // Use get_display_media (no constraints version)
                let promise = match media_devices.get_display_media() {
                    Ok(p) => p,
                    Err(e) => {
                        set_screen_status.set(format!("Failed to request screen: {:?}", e));
                        return;
                    }
                };

                match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(stream) => {
                        let stream: web_sys::MediaStream = stream.unchecked_into();

                        // Get video element and set stream
                        let document = window.document().unwrap();
                        if let Some(video) = document.get_element_by_id("screen-video") {
                            let video: web_sys::HtmlVideoElement = video.unchecked_into();
                            video.set_src_object(Some(&stream));
                            let _ = video.play();
                        }

                        // Store stream reference
                        *screen_stream_ref_inner.borrow_mut() = Some(stream);
                        set_is_screen_sharing.set(true);
                        set_screen_status.set("Screen sharing active".to_string());
                    }
                    Err(e) => {
                        let msg = format!("{:?}", e);
                        set_screen_status.set(format!("Screen sharing failed: {}", msg));
                    }
                }
            });
        }
    };

    let screen_stream_ref_stop = screen_stream_ref.clone();
    let stop_screen_share = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;

            if let Some(stream) = screen_stream_ref_stop.borrow_mut().take() {
                // Stop all tracks using get_video_tracks
                let video_tracks = stream.get_video_tracks();
                for i in 0..video_tracks.length() {
                    let track = video_tracks.get(i);
                    // Call stop() method via js_sys
                    if let Ok(stop_fn) = js_sys::Reflect::get(&track, &"stop".into()) {
                        if let Some(func) = stop_fn.dyn_ref::<js_sys::Function>() {
                            let _ = func.call0(&track);
                        }
                    }
                }
            }

            // Clear video element
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            if let Some(video) = document.get_element_by_id("screen-video") {
                let video: web_sys::HtmlVideoElement = video.unchecked_into();
                video.set_src_object(None);
            }

            set_is_screen_sharing.set(false);
            set_screen_status.set("Screen sharing stopped".to_string());
        }
    };

    let fullscreen_screen = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            if let Some(video) = document.get_element_by_id("screen-video") {
                let video: web_sys::HtmlVideoElement = video.unchecked_into();
                let _ = video.request_fullscreen();
            }
        }
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
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50/30">
            <div class="flex flex-col">
                <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                    <StatusBadge message=status status_type=status_type />
                </Header>

                <NavBar active="remote" />

                <main class="flex-1 px-8 py-10 overflow-y-auto">
                    <div class="max-w-6xl mx-auto">
                        {/* Page Header */}
                        <div class="mb-10">
                            <h1 class="text-3xl font-semibold text-gray-900 tracking-tight mb-3">
                                "Remote Control"
                            </h1>
                            <p class="text-base text-gray-600 leading-relaxed max-w-2xl">
                                "Control your desktop remotely using voice commands, screen sharing, and touch controls."
                            </p>
                        </div>

                        {/* Content Grid */}
                        <div class="space-y-6">
                            <Card title="Screen Sharing">
                                <p class="text-sm text-gray-700 leading-relaxed mb-4">
                                    "View and control your desktop from anywhere"
                                </p>
                    <div id="screen-container" style="margin: 20px 0;">
                        <video
                            id="screen-video"
                            controls
                            autoplay
                            playsinline
                            style="width: 100%; max-width: 100%; border: 1px solid #ddd; display: none; background: #000;"
                            style:display=move || if is_screen_sharing.get() { "block" } else { "none" }
                        ></video>
                        <div
                            id="screen-placeholder"
                            style="width: 100%; height: 300px; background: #f8f9fa; border: 2px dashed #dee2e6; display: flex; align-items: center; justify-content: center; color: #6c757d; border-radius: 8px;"
                            style:display=move || if is_screen_sharing.get() { "none" } else { "flex" }
                        >
                            <div style="text-align: center;">
                                <div style="font-size: 48px; margin-bottom: 10px;">"📺"</div>
                                <div>"Screen sharing not started"</div>
                                <div style="font-size: 14px; margin-top: 5px;">"Click \"Start Screen Share\" to begin"</div>
                            </div>
                        </div>
                    </div>
                    <div class="status info" style="margin-bottom: 15px;">
                        {move || screen_status.get()}
                    </div>
                    <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                        <button
                            class="btn btn-success"
                            on:click=start_screen_share
                            style:display=move || if is_screen_sharing.get() { "none" } else { "inline-block" }
                        >
                            "📺 Start Screen Share"
                        </button>
                        <button
                            class="btn btn-danger"
                            on:click=stop_screen_share
                            style:display=move || if is_screen_sharing.get() { "inline-block" } else { "none" }
                        >
                            "⏹️ Stop Screen Share"
                        </button>
                        <button
                            class="btn btn-secondary"
                            on:click=fullscreen_screen
                            style:display=move || if is_screen_sharing.get() { "inline-block" } else { "none" }
                        >
                            "⛶ Fullscreen"
                        </button>
                    </div>
                </Card>

                            <Card title="Voice Control">
                                <p class="text-sm text-gray-700 leading-relaxed mb-4">
                                    "Control your desktop with voice commands from mobile"
                                </p>
                                <div class="space-y-4">
                                    <h4 class="text-sm font-semibold text-gray-900">"Recent Commands"</h4>
                                    <div class="max-h-48 overflow-y-auto bg-slate-50/50 rounded-xl p-4 border border-slate-200/60">
                                        <Show
                                            when=move || !commands_history.get().is_empty()
                                            fallback=|| view! { <p class="text-sm text-gray-500 italic">"No commands yet"</p> }
                                        >
                                            <div class="space-y-2">
                                                <For
                                                    each=move || commands_history.get()
                                                    key=|cmd| cmd.clone()
                                                    children=move |cmd| view! {
                                                        <p class="text-sm text-gray-700 border-b border-slate-100 pb-2 last:border-b-0">
                                                            {cmd}
                                                        </p>
                                                    }
                                                />
                                            </div>
                                        </Show>
                                    </div>
                                </div>
                            </Card>

                            <Card title="Touch Controls">
                                <p class="text-sm text-gray-700 leading-relaxed mb-4">
                                    "Touch and gesture controls for mobile devices"
                                </p>
                                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                                    <For
                                        each=move || touch_commands.clone()
                                        key=|(label, _)| label.to_string()
                                        children=move |(label, command)| {
                                            let cmd = command.to_string();
                                            let exec = execute_command.clone();
                                            view! {
                                                <button
                                                    class="px-4 py-3 rounded-xl text-sm font-semibold text-white bg-gradient-to-r from-indigo-600 to-indigo-700 hover:from-indigo-700 hover:to-indigo-800 shadow-md shadow-indigo-200/50 hover:shadow-lg hover:shadow-indigo-300/50 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:ring-offset-2"
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
                                <p class="text-sm text-gray-700 leading-relaxed mb-4">
                                    "Type anywhere without a keyboard using voice dictation"
                                </p>
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
                </main>
            </div>
        </div>
    }
}
