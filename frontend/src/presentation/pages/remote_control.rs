//! Remote Control page component - Ultra Premium Design

use crate::infrastructure::api_client as api;
use crate::presentation::components::{Button, ButtonVariant, Header, Icon, IconType, NavBar};
use gloo_timers::future::TimeoutFuture;
use leptos::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures;
use web_sys::{MediaStream, SpeechRecognition, SpeechRecognitionEvent};

#[component]
pub fn RemoteControl() -> impl IntoView {
    let (status, set_status) = create_signal("Ready".to_string());
    let (status_type, set_status_type) = create_signal("info".to_string());
    let (dictation_text, set_dictation_text) = create_signal(String::new());
    let (dictation_status, set_dictation_status) = create_signal("Dictation ready".to_string());
    // Use Rc<RefCell> for immediate synchronous tracking (signals batch updates)
    let last_processed_index: Rc<RefCell<i32>> = Rc::new(RefCell::new(-1));
    let (is_dictating, set_is_dictating) = create_signal(false);

    // Voice commands state
    let (voice_command_text, set_voice_command_text) = create_signal(String::new());
    let (voice_command_status, set_voice_command_status) =
        create_signal("Voice commands ready".to_string());
    let (is_voice_listening, set_is_voice_listening) = create_signal(false);
    let (commands_history, set_commands_history) = create_signal::<Vec<String>>(vec![]);

    // Screen sharing state
    let (screen_status, set_screen_status) =
        create_signal("Click to start screen sharing".to_string());
    let (is_screen_sharing, set_is_screen_sharing) = create_signal(false);

    // Store reference to MediaStream
    let screen_stream_ref: Rc<RefCell<Option<MediaStream>>> = Rc::new(RefCell::new(None));
    let screen_stream_for_stop = screen_stream_ref.clone();

    // Store reference to SpeechRecognition instance
    let recognition_ref: Rc<RefCell<Option<SpeechRecognition>>> = Rc::new(RefCell::new(None));
    let recognition_for_stop = recognition_ref.clone();

    let execute_command = move |command: String| {
        // Add to history
        set_commands_history.update(|history| {
            history.insert(0, command.clone());
            if history.len() > 10 {
                history.pop();
            }
        });

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
                }
                Err(e) => {
                    set_status.set(format!("Error: {}", e));
                    set_status_type.set("error".to_string());
                }
            }
        });
    };

    let last_idx_for_clear = last_processed_index.clone();
    let clear_dictation = move |_| {
        set_dictation_text.set(String::new());
        *last_idx_for_clear.borrow_mut() = -1;
        set_dictation_status.set("Dictation cleared".to_string());
    };

    let test_keyboard = move |_| {
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            set_dictation_status.set("Testing keyboard simulation...".to_string());
            match api::ApiClient::new_default().test_keyboard().await {
                Ok(_) => {
                    set_dictation_status.set(
                        "Keyboard test completed - check your desktop for typed text!".to_string(),
                    );
                }
                Err(e) => {
                    set_dictation_status.set(format!("Keyboard test failed: {}", e));
                }
            }
        });
    };

    // Real-time dictation with Web Speech API
    let last_idx_for_start = last_processed_index.clone();
    let start_dictation = {
        let recognition_ref = recognition_ref.clone();
        move |_| {
            #[cfg(target_arch = "wasm32")]
            {
                // Check if SpeechRecognition is available
                let window = web_sys::window().expect("no global window");
                let speech_recognition =
                    js_sys::Reflect::get(&window, &"webkitSpeechRecognition".into())
                        .or_else(|_| js_sys::Reflect::get(&window, &"SpeechRecognition".into()));

                match speech_recognition {
                    Ok(sr_constructor) if !sr_constructor.is_undefined() => {
                        let sr_constructor: js_sys::Function = sr_constructor.unchecked_into();
                        let recognition: SpeechRecognition =
                            js_sys::Reflect::construct(&sr_constructor, &js_sys::Array::new())
                                .expect("Failed to create SpeechRecognition")
                                .unchecked_into();

                        // Configure recognition - interim results for real-time feedback
                        recognition.set_continuous(true);
                        recognition.set_interim_results(true);
                        recognition.set_lang("en-US");

                        // Set up result handler - real-time display, type only finals
                        let set_text = set_dictation_text.clone();
                        let set_status_inner = set_dictation_status.clone();
                        let last_idx = last_idx_for_start.clone();
                        let on_result = Closure::wrap(Box::new(
                            move |event: SpeechRecognitionEvent| {
                                if let Some(results) = event.results() {
                                    let result_index = event.result_index();

                                    if let Some(result) = results.get(result_index) {
                                        let alternative = result.item(0);
                                        let text = alternative.transcript();
                                        let confidence = alternative.confidence();

                                        // Always show current text for real-time feedback
                                        set_text.set(text.clone());

                                        if !result.is_final() {
                                            // Show interim result status
                                            set_status_inner.set(format!("Hearing: \"{}\"", text.trim()));
                                            return;
                                        }

                                        // FINAL result - check if already typed
                                        let idx = result_index as i32;
                                        {
                                            let current = *last_idx.borrow();
                                            if idx <= current {
                                                return; // Already typed this
                                            }
                                            *last_idx.borrow_mut() = idx;
                                        }

                                        // Skip low confidence
                                        if confidence < 0.5 {
                                            set_status_inner.set(format!(
                                                "Low confidence ({:.0}%): \"{}\"",
                                                confidence * 100.0, text.trim()
                                            ));
                                            return;
                                        }

                                        // Skip empty
                                        if text.trim().is_empty() {
                                            return;
                                        }

                                        // Type the final text
                                        let text_to_type = format!("{} ", text);
                                        let set_status = set_status_inner.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            match api::ApiClient::new_default()
                                                .type_dictation(&text_to_type)
                                                .await
                                            {
                                                Ok(response) => {
                                                    if response.success {
                                                        set_status.set(format!(
                                                            "Typed: \"{}\"",
                                                            text.trim()
                                                        ));
                                                    } else {
                                                        set_status.set("Typing failed".to_string());
                                                    }
                                                }
                                                Err(_) => {
                                                    set_status.set("Network error".to_string());
                                                }
                                            }
                                        });
                                    }
                                }
                            },
                        )
                            as Box<dyn FnMut(_)>);

                        recognition.set_onresult(Some(on_result.as_ref().unchecked_ref()));
                        on_result.forget();

                        // Set up error handler
                        let set_status_err = set_dictation_status.clone();
                        let set_is_dict = set_is_dictating.clone();
                        let on_error = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                            set_status_err.set("Speech recognition error occurred".to_string());
                            set_is_dict.set(false);
                        })
                            as Box<dyn FnMut(_)>);

                        recognition.set_onerror(Some(on_error.as_ref().unchecked_ref()));
                        on_error.forget();

                        // Set up end handler
                        let set_status_end = set_dictation_status.clone();
                        let set_is_dict_end = set_is_dictating.clone();
                        let on_end = Closure::wrap(Box::new(move || {
                            set_status_end.set("Dictation stopped".to_string());
                            set_is_dict_end.set(false);
                        }) as Box<dyn FnMut()>);

                        recognition.set_onend(Some(on_end.as_ref().unchecked_ref()));
                        on_end.forget();

                        // Start recognition
                        let _ = recognition.start();
                        *recognition_ref.borrow_mut() = Some(recognition);

                        set_dictation_status.set("Listening... speak now".to_string());
                        set_is_dictating.set(true);
                    }
                    _ => {
                        set_dictation_status.set(
                            "Speech recognition not supported in this browser. Try Chrome or Edge."
                                .to_string(),
                        );
                    }
                }
            }
        }
    };

    let stop_dictation = {
        let recognition_ref = recognition_ref.clone();
        let last_idx_for_stop = last_processed_index.clone();
        move |_| {
            if let Some(recognition) = &*recognition_ref.borrow() {
                let _ = recognition.stop();
            }
            *last_idx_for_stop.borrow_mut() = -1;
            set_dictation_status.set("Dictation stopped".to_string());
            set_is_dictating.set(false);
        }
    };

    // Voice command processing
    let start_voice_commands = {
        let set_text = set_voice_command_text.clone();
        let set_status = set_voice_command_status.clone();
        let execute_command = execute_command.clone();
        let recognition_ref = recognition_ref.clone();
        move |_| {
            set_is_voice_listening.set(true);
            set_voice_command_status.set("Listening for voice commands...".to_string());

            #[cfg(target_arch = "wasm32")]
            {
                // Check if SpeechRecognition is available
                let window = web_sys::window().expect("no global window");
                let speech_recognition =
                    js_sys::Reflect::get(&window, &"webkitSpeechRecognition".into())
                        .or_else(|_| js_sys::Reflect::get(&window, &"SpeechRecognition".into()));

                match speech_recognition {
                    Ok(sr_constructor) if !sr_constructor.is_undefined() => {
                        let sr_constructor: js_sys::Function = sr_constructor.unchecked_into();
                        let recognition: SpeechRecognition =
                            js_sys::Reflect::construct(&sr_constructor, &js_sys::Array::new())
                                .expect("Failed to create SpeechRecognition")
                                .unchecked_into();

                        // Configure recognition for commands - continuous mode for always listening
                        recognition.set_continuous(true);
                        recognition.set_interim_results(false);
                        recognition.set_lang("en-US");
                        recognition.set_max_alternatives(3);

                        // Set up result handler for voice commands
                        let set_text = set_text.clone();
                        let set_status = set_status.clone();
                        let execute_command = execute_command.clone();
                        let is_voice_listening_for_result = is_voice_listening.clone();
                        let recognition_ref_for_result = recognition_ref.clone();
                        let on_result =
                            Closure::wrap(Box::new(move |event: SpeechRecognitionEvent| {
                                if let Some(results) = event.results() {
                                    // Get the latest result (last in the list)
                                    let result_index = event.result_index();
                                    if let Some(result) = results.get(result_index) {
                                        // Only process final results
                                        if !result.is_final() {
                                            return;
                                        }

                                        let alternative = result.item(0);
                                        let confidence = alternative.confidence();
                                        let command_text = alternative.transcript().trim().to_string();

                                        // Confidence threshold - reject low confidence commands
                                        if confidence < 0.6 {
                                            set_status.set(format!(
                                                "Low confidence ({:.0}%): \"{}\" - please repeat",
                                                confidence * 100.0,
                                                command_text
                                            ));
                                            return;
                                        }

                                        // Reject very short or empty commands
                                        if command_text.len() < 2 {
                                            return;
                                        }

                                        set_text.set(command_text.clone());

                                        // Process the voice command
                                        let set_text = set_text.clone();
                                        let set_status = set_status.clone();
                                        let confidence = confidence;
                                        wasm_bindgen_futures::spawn_local(async move {
                                            set_status.set(format!(
                                                "Processing ({:.0}%): \"{}\"",
                                                confidence * 100.0,
                                                command_text
                                            ));
                                            match api::ApiClient::new_default()
                                                .process_voice_command(
                                                    &command_text,
                                                    Some(confidence),
                                                )
                                                .await
                                            {
                                                Ok(response) => {
                                                    if response
                                                        .get("success")
                                                        .unwrap_or(&serde_json::Value::Bool(
                                                            false,
                                                        ))
                                                        .as_bool()
                                                        .unwrap_or(false)
                                                    {
                                                        set_status.set(format!(
                                                            "✓ {}",
                                                            response
                                                                .get("message")
                                                                .unwrap_or(
                                                                    &serde_json::Value::String(
                                                                        "Command executed"
                                                                            .to_string()
                                                                    )
                                                                )
                                                                .as_str()
                                                                .unwrap_or("Command executed")
                                                        ));
                                                    } else {
                                                        set_status.set(format!(
                                                            "✗ {}",
                                                            response
                                                                .get("message")
                                                                .unwrap_or(
                                                                    &serde_json::Value::String(
                                                                        "Command failed"
                                                                            .to_string()
                                                                    )
                                                                )
                                                                .as_str()
                                                                .unwrap_or("Command failed")
                                                        ));
                                                    }
                                                }
                                                Err(e) => {
                                                    set_status.set(format!("Error: {}", e));
                                                }
                                            }

                                            // Clear command text after processing
                                            set_text.set(String::new());

                                            // Update status to show we're still listening
                                            TimeoutFuture::new(1_500).await;
                                            set_status.set("Listening for voice commands...".to_string());
                                        });
                                    }
                                }
                            }) as Box<dyn FnMut(_)>);

                        recognition.set_onresult(Some(on_result.as_ref().unchecked_ref()));
                        on_result.forget();

                        let set_status_err = set_status.clone();
                        let is_voice_listening_err = is_voice_listening.clone();
                        let recognition_ref_err = recognition_ref.clone();
                        recognition.set_onerror(Some(
                            Closure::wrap(Box::new(move |_: web_sys::Event| {
                                set_status_err.set("Voice recognition error - restarting...".to_string());
                                // Auto-restart on error if still in listening mode
                                if is_voice_listening_err.get() {
                                    if let Some(recognition) = &*recognition_ref_err.borrow() {
                                        let _ = recognition.start();
                                    }
                                }
                            }) as Box<dyn FnMut(_)>)
                            .as_ref()
                            .unchecked_ref(),
                        ));

                        let set_status_end = set_status.clone();
                        let is_voice_listening_end = is_voice_listening.clone();
                        let recognition_ref_end = recognition_ref.clone();
                        recognition.set_onend(Some(
                            Closure::wrap(Box::new(move |_: web_sys::Event| {
                                // Auto-restart if still in listening mode (continuous listening)
                                if is_voice_listening_end.get() {
                                    set_status_end.set("Restarting voice recognition...".to_string());
                                    if let Some(recognition) = &*recognition_ref_end.borrow() {
                                        let _ = recognition.start();
                                    }
                                } else {
                                    set_status_end.set("Voice commands stopped".to_string());
                                }
                            }) as Box<dyn FnMut(_)>)
                            .as_ref()
                            .unchecked_ref(),
                        ));

                        // Start recognition
                        let _ = recognition.start();
                        *recognition_ref.borrow_mut() = Some(recognition);
                    }
                    _ => {
                        set_status.set(
                            "Speech recognition not supported in this browser. Try Chrome or Edge."
                                .to_string(),
                        );
                    }
                }
            }
        }
    };

    let stop_voice_commands = {
        let recognition_ref = recognition_ref.clone();
        move |_| {
            set_is_voice_listening.set(false);
            set_voice_command_status.set("Voice commands stopped".to_string());
            if let Some(recognition) = &*recognition_ref.borrow() {
                let _ = recognition.stop();
            }
        }
    };

    // Screen sharing with getDisplayMedia
    let start_screen_share = {
        let stream_ref = screen_stream_ref.clone();
        move |_| {
            #[cfg(target_arch = "wasm32")]
            {
                let stream_ref = stream_ref.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let window = web_sys::window().expect("no global window");
                    let navigator = window.navigator();
                    let media_devices = match navigator.media_devices() {
                        Ok(md) => md,
                        Err(e) => {
                            set_screen_status.set(format!("No media devices: {:?}", e));
                            return;
                        }
                    };

                    // Use get_display_media without constraints
                    match media_devices.get_display_media() {
                        Ok(promise) => {
                            let future = wasm_bindgen_futures::JsFuture::from(promise);
                            match future.await {
                                Ok(stream_value) => {
                                    let stream: MediaStream = stream_value.unchecked_into();

                                    // Get video element and attach stream
                                    let document = window.document().expect("no document");
                                    if let Some(video_elem) =
                                        document.get_element_by_id("screen-video")
                                    {
                                        let video: web_sys::HtmlVideoElement =
                                            video_elem.unchecked_into();
                                        video.set_src_object(Some(&stream));
                                        let _ = video.play();
                                    }

                                    // Store stream reference for cleanup
                                    *stream_ref.borrow_mut() = Some(stream);

                                    set_screen_status.set("Screen sharing active".to_string());
                                    set_is_screen_sharing.set(true);
                                }
                                Err(e) => {
                                    set_screen_status.set(format!(
                                        "Failed to start: {:?}",
                                        e.as_string().unwrap_or_default()
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            set_screen_status.set(format!("Screen capture error: {:?}", e));
                        }
                    }
                });
            }
        }
    };

    let stop_screen_share = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            // Stop all tracks
            if let Some(stream) = screen_stream_for_stop.borrow_mut().take() {
                let video_tracks = stream.get_video_tracks();
                for i in 0..video_tracks.length() {
                    let track = video_tracks.get(i);
                    let _ = js_sys::Reflect::apply(
                        &js_sys::Reflect::get(&track, &"stop".into())
                            .unwrap()
                            .unchecked_into(),
                        &track,
                        &js_sys::Array::new(),
                    );
                }
            }

            // Clear video element
            let window = web_sys::window().expect("no global window");
            let document = window.document().expect("no document");
            if let Some(video_elem) = document.get_element_by_id("screen-video") {
                let video: web_sys::HtmlVideoElement = video_elem.unchecked_into();
                video.set_src_object(None);
            }
        }

        set_is_screen_sharing.set(false);
        set_screen_status.set("Screen sharing stopped".to_string());
    };

    let enter_fullscreen = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().expect("no global window");
            let document = window.document().expect("no document");
            if let Some(video_elem) = document.get_element_by_id("screen-video") {
                let video: web_sys::HtmlVideoElement = video_elem.unchecked_into();
                let _ = video.request_fullscreen();
            }
        }
    };

    let touch_commands = vec![
        ("Terminal", "open terminal", IconType::Commands),
        ("Browser", "open browser", IconType::Screen),
        ("Vol Up", "volume up", IconType::Workflows),
        ("Vol Down", "volume down", IconType::Workflows),
        ("Next WS", "workspace next", IconType::Home),
        ("Prev WS", "workspace previous", IconType::Home),
        ("Close Win", "window close", IconType::X),
        ("Screenshot", "take screenshot", IconType::Screen),
    ];

    view! {
        <div class="min-h-screen bg-gradient-to-br from-slate-50 via-white to-slate-50/30">
            <div class="flex flex-col">
                <Header title="Vibespeak" subtitle="Voice Automation System" />

                <NavBar _active="remote" />

                <main class="flex-1 px-4 sm:px-6 lg:px-8 py-4 sm:py-6 lg:py-10 overflow-y-auto">
                    <div class="max-w-6xl mx-auto">
                        // Page Header
                        <div class="mb-8 sm:mb-10">
                            <p class="text-xs font-semibold tracking-wide uppercase text-indigo-600 mb-2">
                                "Control Center"
                            </p>
                            <h1 class="text-2xl sm:text-3xl font-semibold text-gray-900 tracking-tight mb-3">
                                "Remote Control"
                            </h1>
                            <p class="text-sm sm:text-base text-gray-600 leading-relaxed max-w-2xl">
                                "Control your desktop remotely using voice commands, screen sharing, and touch controls."
                            </p>
                        </div>

                        // Content Grid
                        <div class="space-y-6">
                            // Screen Sharing Card
                            <section class="relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 overflow-hidden transition-all duration-300 hover:shadow-md hover:shadow-slate-100/60">
                                <div class="px-6 sm:px-7 py-5 border-b border-slate-100 bg-gradient-to-r from-white to-slate-50/30">
                                    <div class="flex items-center justify-between">
                                        <div class="flex items-center gap-3">
                                            <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-indigo-600 flex items-center justify-center shadow-md shadow-indigo-200/50">
                                                <Icon
                                                    icon_type=IconType::Screen
                                                    class="w-5 h-5 text-white"
                                                />
                                            </div>
                                            <div>
                                                <h3 class="text-lg font-semibold text-gray-900 tracking-tight">
                                                    "Screen Sharing"
                                                </h3>
                                                <p class="text-xs text-slate-500">
                                                    "Share your screen in real-time"
                                                </p>
                                            </div>
                                        </div>
                                        <div class="flex items-center gap-2">
                                            {move || {
                                                if is_screen_sharing.get() {
                                                    view! {
                                                        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-emerald-50 text-emerald-700">
                                                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                                                            "Live"
                                                        </span>
                                                    }
                                                        .into_view()
                                                } else {
                                                    view! {
                                                        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-slate-100 text-slate-600">
                                                            <span class="w-1.5 h-1.5 rounded-full bg-slate-400"></span>
                                                            "Inactive"
                                                        </span>
                                                    }
                                                        .into_view()
                                                }
                                            }}
                                        </div>
                                    </div>
                                </div>
                                <div class="px-6 sm:px-7 py-6 bg-gradient-to-b from-white to-slate-50/30">
                                    // Video Container
                                    <div class="relative mb-4 rounded-xl overflow-hidden bg-slate-900 shadow-inner">
                                        <video
                                            id="screen-video"
                                            autoplay
                                            playsinline
                                            muted
                                            class="w-full aspect-video object-contain"
                                            style:display=move || {
                                                if is_screen_sharing.get() { "block" } else { "none" }
                                            }
                                        ></video>
                                        <div
                                            class="w-full aspect-video flex items-center justify-center bg-gradient-to-br from-slate-800 to-slate-900"
                                            style:display=move || {
                                                if is_screen_sharing.get() { "none" } else { "flex" }
                                            }
                                        >
                                            <div class="text-center space-y-4 p-8">
                                                <div class="w-16 h-16 mx-auto rounded-2xl bg-slate-700/50 flex items-center justify-center">
                                                    <Icon
                                                        icon_type=IconType::Screen
                                                        class="w-8 h-8 text-slate-400"
                                                    />
                                                </div>
                                                <div class="space-y-2">
                                                    <p class="text-sm font-medium text-slate-300">
                                                        "No screen shared"
                                                    </p>
                                                    <p class="text-xs text-slate-500">
                                                        "Click the button below to share your screen"
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // Status
                                    <div class="mb-4 px-4 py-2.5 rounded-xl bg-slate-50 border border-slate-200/60">
                                        <p class="text-sm text-slate-600">
                                            {move || screen_status.get()}
                                        </p>
                                    </div>

                                    // Controls
                                    <div class="flex flex-wrap gap-3">
                                        {
                                            let start_screen_share = StoredValue::new(
                                                start_screen_share,
                                            );
                                            let stop_screen_share = StoredValue::new(stop_screen_share);
                                            let enter_fullscreen = StoredValue::new(enter_fullscreen);
                                            move || {
                                                if !is_screen_sharing.get() {
                                                    view! {
                                                        <Button
                                                            variant=ButtonVariant::Primary
                                                            on:click=start_screen_share.get_value()
                                                        >
                                                            <Icon icon_type=IconType::Screen class="w-4 h-4 mr-2" />
                                                            "Start Sharing"
                                                        </Button>
                                                    }
                                                        .into_view()
                                                } else {
                                                    view! {
                                                        <>
                                                            <Button
                                                                variant=ButtonVariant::Danger
                                                                on:click=stop_screen_share.get_value()
                                                            >
                                                                <Icon icon_type=IconType::Stop class="w-4 h-4 mr-2" />
                                                                "Stop"
                                                            </Button>
                                                            <Button
                                                                variant=ButtonVariant::Secondary
                                                                on:click=enter_fullscreen.get_value()
                                                            >
                                                                <Icon icon_type=IconType::Fullscreen class="w-4 h-4 mr-2" />
                                                                "Fullscreen"
                                                            </Button>
                                                        </>
                                                    }
                                                        .into_view()
                                                }
                                            }
                                        }
                                    </div>
                                </div>
                            </section>

                            // Voice Dictation Card
                            <section class="relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 overflow-hidden transition-all duration-300 hover:shadow-md hover:shadow-slate-100/60">
                                <div class="px-6 sm:px-7 py-5 border-b border-slate-100 bg-gradient-to-r from-white to-slate-50/30">
                                    <div class="flex items-center justify-between">
                                        <div class="flex items-center gap-3">
                                            <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-rose-500 to-rose-600 flex items-center justify-center shadow-md shadow-rose-200/50">
                                                <Icon
                                                    icon_type=IconType::Microphone
                                                    class="w-5 h-5 text-white"
                                                />
                                            </div>
                                            <div>
                                                <h3 class="text-lg font-semibold text-gray-900 tracking-tight">
                                                    "Voice Dictation"
                                                </h3>
                                                <p class="text-xs text-slate-500">
                                                    "Speak and type anywhere"
                                                </p>
                                            </div>
                                        </div>
                                        <div class="flex items-center gap-2">
                                            {move || {
                                                if is_dictating.get() {
                                                    view! {
                                                        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-rose-50 text-rose-700">
                                                            <span class="w-1.5 h-1.5 rounded-full bg-rose-500 animate-pulse"></span>
                                                            "Recording"
                                                        </span>
                                                    }
                                                        .into_view()
                                                } else {
                                                    view! {
                                                        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-slate-100 text-slate-600">
                                                            <span class="w-1.5 h-1.5 rounded-full bg-slate-400"></span>
                                                            "Ready"
                                                        </span>
                                                    }
                                                        .into_view()
                                                }
                                            }}
                                        </div>
                                    </div>
                                </div>
                                <div class="px-6 sm:px-7 py-6 bg-gradient-to-b from-white to-slate-50/30">
                                    // Status
                                    <div class="mb-4 px-4 py-2.5 rounded-xl bg-slate-50 border border-slate-200/60">
                                        <p class="text-sm text-slate-600">
                                            {move || dictation_status.get()}
                                        </p>
                                    </div>

                                    // Text Area
                                    <div class="mb-4">
                                        <textarea
                                            class="w-full px-4 py-3 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200 resize-none min-h-[120px]"
                                            placeholder="Dictated text will appear here and be typed automatically... Click 'Start Dictation' and speak."
                                            prop:value=move || dictation_text.get()
                                            on:input=move |ev| {
                                                set_dictation_text.set(event_target_value(&ev));
                                            }
                                        ></textarea>
                                    </div>

                                    // Controls
                                    <div class="flex flex-wrap gap-3">
                                        {
                                            let start_dictation = StoredValue::new(start_dictation);
                                            let stop_dictation = StoredValue::new(stop_dictation);
                                            move || {
                                                if !is_dictating.get() {
                                                    view! {
                                                        <Button
                                                            variant=ButtonVariant::Primary
                                                            on:click=start_dictation.get_value()
                                                        >
                                                            <Icon icon_type=IconType::Microphone class="w-4 h-4 mr-2" />
                                                            "Start Dictation"
                                                        </Button>
                                                    }
                                                        .into_view()
                                                } else {
                                                    view! {
                                                        <Button
                                                            variant=ButtonVariant::Danger
                                                            on:click=stop_dictation.get_value()
                                                        >
                                                            <Icon icon_type=IconType::Stop class="w-4 h-4 mr-2" />
                                                            "Stop"
                                                        </Button>
                                                    }
                                                        .into_view()
                                                }
                                            }
                                        }
                                        <Button
                                            variant=ButtonVariant::Ghost
                                            on:click=clear_dictation
                                        >
                                            <Icon icon_type=IconType::X class="w-4 h-4 mr-2" />
                                            "Clear"
                                        </Button>
                                    </div>

                                    // Info Box
                                    <div class="mt-4 p-4 bg-indigo-50/50 rounded-xl border border-indigo-100">
                                        <div class="flex gap-3">
                                            <Icon
                                                icon_type=IconType::Microphone
                                                class="w-4 h-4 text-indigo-600 flex-shrink-0 mt-0.5"
                                            />
                                            <p class="text-xs text-indigo-700 leading-relaxed">
                                                "Speak naturally and your words will appear above. Click 'Send to Desktop' to type the text into your focused application."
                                            </p>
                                        </div>
                                    </div>
                                 </div>
                             </section>

                             // Voice Commands Card
                             <section class="relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 overflow-hidden transition-all duration-300 hover:shadow-md hover:shadow-slate-100/60">
                                 <div class="px-6 sm:px-7 py-5 border-b border-slate-100 bg-gradient-to-r from-white to-slate-50/30">
                                     <div class="flex items-center justify-between">
                                         <div class="flex items-center gap-3">
                                             <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-purple-500 to-purple-600 flex items-center justify-center shadow-md shadow-purple-200/50">
                                                 <Icon
                                                     icon_type=IconType::Microphone
                                                     class="w-5 h-5 text-white"
                                                 />
                                             </div>
                                             <div>
                                                 <h3 class="text-lg font-semibold text-gray-900 tracking-tight">
                                                     "Voice Commands"
                                                 </h3>
                                                 <p class="text-xs text-slate-500">
                                                     "Execute voice commands directly"
                                                 </p>
                                             </div>
                                         </div>
                                         <div class="flex items-center gap-2">
                                             {move || {
                                                 if is_voice_listening.get() {
                                                     view! {
                                                         <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-emerald-50 text-emerald-700">
                                                             <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                                                             "Listening"
                                                         </span>
                                                     }
                                                         .into_view()
                                                 } else {
                                                     view! {
                                                         <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-medium bg-slate-100 text-slate-600">
                                                             <span class="w-1.5 h-1.5 rounded-full bg-slate-400"></span>
                                                             "Ready"
                                                         </span>
                                                     }
                                                         .into_view()
                                                 }
                                             }}
                                         </div>
                                     </div>
                                 </div>
                                 <div class="px-6 sm:px-7 py-6 bg-gradient-to-b from-white to-slate-50/30">
                                     // Last Command
                                     <div class="mb-4">
                                         <label class="block text-sm font-medium text-gray-700 mb-2">
                                             "Last Command"
                                         </label>
                                         <input
                                             type="text"
                                             readonly
                                             class="w-full px-3 py-2 rounded-lg border border-slate-200 bg-slate-50 text-sm text-slate-600 focus:outline-none focus:ring-0"
                                             prop:value=move || voice_command_text.get()
                                         />
                                     </div>

                                     // Status
                                     <div class="mb-4 px-4 py-2.5 rounded-xl bg-slate-50 border border-slate-200/60">
                                         <p class="text-sm text-slate-600">
                                             {move || voice_command_status.get()}
                                         </p>
                                     </div>

                                     // Controls
                                     <div class="flex flex-wrap gap-3">
                                         {
                                             let start_voice_commands = StoredValue::new(start_voice_commands);
                                             let stop_voice_commands = StoredValue::new(stop_voice_commands);
                                             move || {
                                                 if !is_voice_listening.get() {
                                                     view! {
                                                         <Button
                                                             variant=ButtonVariant::Primary
                                                             on:click=start_voice_commands.get_value()
                                                         >
                                                             <Icon icon_type=IconType::Microphone class="w-4 h-4 mr-2" />
                                                             "Listen for Command"
                                                         </Button>
                                                     }
                                                         .into_view()
                                                 } else {
                                                     view! {
                                                         <Button
                                                             variant=ButtonVariant::Danger
                                                             on:click=stop_voice_commands.get_value()
                                                         >
                                                             <Icon icon_type=IconType::Stop class="w-4 h-4 mr-2" />
                                                             "Stop Listening"
                                                         </Button>
                                                     }
                                                         .into_view()
                                                 }
                                             }
                                         }
                                     </div>

                                     // Info Box
                                     <div class="mt-4 p-4 bg-purple-50/50 rounded-xl border border-purple-100">
                                         <div class="flex gap-3">
                                             <Icon
                                                 icon_type=IconType::Microphone
                                                 class="w-4 h-4 text-purple-600 flex-shrink-0 mt-0.5"
                                             />
                                             <p class="text-xs text-purple-700 leading-relaxed">
                                                 "Say a voice command like 'open browser' or 'new window'. Commands are processed instantly when recognized."
                                             </p>
                                         </div>
                                     </div>
                                 </div>
                             </section>

                             // Two-column layout for smaller cards
                            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                // Touch Controls Card
                                <section class="relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 overflow-hidden transition-all duration-300 hover:shadow-md hover:shadow-slate-100/60">
                                    <div class="px-6 sm:px-7 py-5 border-b border-slate-100 bg-gradient-to-r from-white to-slate-50/30">
                                        <div class="flex items-center gap-3">
                                            <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-amber-500 to-amber-600 flex items-center justify-center shadow-md shadow-amber-200/50">
                                                <Icon
                                                    icon_type=IconType::Commands
                                                    class="w-5 h-5 text-white"
                                                />
                                            </div>
                                            <div>
                                                <h3 class="text-lg font-semibold text-gray-900 tracking-tight">
                                                    "Quick Actions"
                                                </h3>
                                                <p class="text-xs text-slate-500">"One-tap controls"</p>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="px-6 sm:px-7 py-6 bg-gradient-to-b from-white to-slate-50/30">
                                        <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                                            {touch_commands
                                                .into_iter()
                                                .map(|(label, command, icon)| {
                                                    let cmd = command.to_string();
                                                    let exec = execute_command.clone();
                                                    view! {
                                                        <button
                                                            class="group relative flex flex-col items-center justify-center gap-2 px-3 py-4 rounded-xl text-sm font-medium text-slate-700 bg-slate-50 border border-slate-200/60 hover:bg-gradient-to-br hover:from-indigo-500 hover:to-indigo-600 hover:text-white hover:border-transparent hover:shadow-lg hover:shadow-indigo-200/50 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:ring-offset-2"
                                                            on:click=move |_| exec(cmd.clone())
                                                        >
                                                            <Icon
                                                                icon_type=icon
                                                                class="w-5 h-5 text-slate-400 group-hover:text-white transition-colors"
                                                            />
                                                            <span class="text-xs">{label}</span>
                                                        </button>
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
                                    </div>
                                </section>

                                // Command History Card
                                <section class="relative rounded-2xl bg-white shadow-sm shadow-slate-100/50 border border-slate-200/60 overflow-hidden transition-all duration-300 hover:shadow-md hover:shadow-slate-100/60">
                                    <div class="px-6 sm:px-7 py-5 border-b border-slate-100 bg-gradient-to-r from-white to-slate-50/30">
                                        <div class="flex items-center gap-3">
                                            <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-emerald-500 to-emerald-600 flex items-center justify-center shadow-md shadow-emerald-200/50">
                                                <Icon
                                                    icon_type=IconType::Workflows
                                                    class="w-5 h-5 text-white"
                                                />
                                            </div>
                                            <div>
                                                <h3 class="text-lg font-semibold text-gray-900 tracking-tight">
                                                    "Recent Commands"
                                                </h3>
                                                <p class="text-xs text-slate-500">"Command history"</p>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="px-6 sm:px-7 py-6 bg-gradient-to-b from-white to-slate-50/30">
                                        <div class="max-h-48 overflow-y-auto">
                                            {move || {
                                                let history = commands_history.get();
                                                if history.is_empty() {
                                                    view! {
                                                        <div class="text-center py-8">
                                                            <div class="w-12 h-12 mx-auto mb-3 rounded-full bg-slate-100 flex items-center justify-center">
                                                                <Icon
                                                                    icon_type=IconType::Workflows
                                                                    class="w-6 h-6 text-slate-400"
                                                                />
                                                            </div>
                                                            <p class="text-sm text-slate-500">"No commands yet"</p>
                                                            <p class="text-xs text-slate-400 mt-1">
                                                                "Execute a command to see it here"
                                                            </p>
                                                        </div>
                                                    }
                                                        .into_view()
                                                } else {
                                                    view! {
                                                        <div class="space-y-2">
                                                            {history
                                                                .into_iter()
                                                                .map(|cmd| {
                                                                    view! {
                                                                        <div class="flex items-center gap-3 px-3 py-2 rounded-lg bg-slate-50/50 border border-slate-100">
                                                                            <div class="w-1.5 h-1.5 rounded-full bg-emerald-500"></div>
                                                                            <code class="text-xs font-mono text-slate-700">{cmd}</code>
                                                                        </div>
                                                                    }
                                                                })
                                                                .collect_view()}
                                                        </div>
                                                    }
                                                        .into_view()
                                                }
                                            }}
                                        </div>
                                    </div>
                                </section>
                            </div>

                            // Test Section
                            <section class="relative rounded-2xl bg-gradient-to-r from-slate-800 to-slate-900 shadow-xl shadow-slate-300/30 overflow-hidden">
                                <div class="px-6 sm:px-7 py-6">
                                    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                                        <div class="flex items-center gap-4">
                                            <div class="w-12 h-12 rounded-xl bg-white/10 flex items-center justify-center">
                                                <Icon
                                                    icon_type=IconType::Keyboard
                                                    class="w-6 h-6 text-white"
                                                />
                                            </div>
                                            <div>
                                                <h3 class="text-lg font-semibold text-white">
                                                    "Test Keyboard"
                                                </h3>
                                                <p class="text-sm text-slate-400">
                                                    "Verify keyboard simulation is working"
                                                </p>
                                            </div>
                                        </div>
                                        <Button
                                            variant=ButtonVariant::Secondary
                                            on:click=test_keyboard
                                        >
                                            <Icon icon_type=IconType::Keyboard class="w-4 h-4 mr-2" />
                                            "Run Test"
                                        </Button>
                                    </div>
                                </div>
                            </section>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    }
}
