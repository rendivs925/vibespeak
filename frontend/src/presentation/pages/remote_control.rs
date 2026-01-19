//! Remote Control page component

use crate::infrastructure::api_client as api;
use crate::presentation::components::{
    Button, ButtonVariant, Card, Header, Icon, IconType, NavBar, StatusBadge,
};
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
    let start_dictation = move |_| {
        set_dictation_status.set("Dictation started".to_string());
        set_is_dictating.set(true);
    };

    let stop_dictation = move |_| {
        set_is_dictating.set(false);
        set_dictation_status.set("Dictation stopped".to_string());
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

                <NavBar _active="remote" />

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
                                    class="w-full h-64 bg-slate-50/50 border-2 border-dashed border-slate-300 rounded-2xl flex items-center justify-center"
                                    style:display=move || if is_screen_sharing.get() { "none" } else { "flex" }
                                >
                                    <div class="text-center space-y-3">
                                        <Icon icon_type=IconType::Screen class="w-12 h-12 text-slate-400 mx-auto" />
                                        <div class="space-y-1">
                                            <p class="text-sm font-medium text-slate-700">"Screen sharing not started"</p>
                                            <p class="text-xs text-slate-500">"Click the button below to begin"</p>
                                        </div>
                                    </div>
                                </div>
                    </div>
                    <div class="status info" style="margin-bottom: 15px;">
                        {move || screen_status.get()}
                    </div>
                    <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                        <div class="flex gap-3">
                            <Show
                                when=move || !is_screen_sharing.get()
                                fallback=|| view! { <div></div> }
                            >
                                <Button variant=ButtonVariant::Primary on:click=move |_| {
                                    set_screen_status.set("Screen sharing started".to_string());
                                    set_is_screen_sharing.set(true);
                                }>
                                    <Icon icon_type=IconType::Screen class="w-4 h-4 mr-2" />
                                    "Start Screen Share"
                                </Button>
                            </Show>
                            <Show
                                when=move || is_screen_sharing.get()
                                fallback=|| view! { <div></div> }
                            >
                                <Button variant=ButtonVariant::Danger on:click=move |_| {
                                    set_is_screen_sharing.set(false);
                                    set_screen_status.set("Screen sharing stopped".to_string());
                                }>
                                    <Icon icon_type=IconType::Stop class="w-4 h-4 mr-2" />
                                    "Stop Screen Share"
                                </Button>
                                <Button variant=ButtonVariant::Secondary on:click=move |_| {
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
                                }>
                                    <Icon icon_type=IconType::Fullscreen class="w-4 h-4 mr-2" />
                                    "Fullscreen"
                                </Button>
                            </Show>
                        </div>
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
                    <div class="flex gap-3">
                        <Button variant=ButtonVariant::Primary on:click=start_dictation>
                            <Icon icon_type=IconType::Microphone class="w-4 h-4 mr-2" />
                            "Dictation Mode"
                        </Button>
                        <Button variant=ButtonVariant::Danger on:click=stop_dictation>
                            <Icon icon_type=IconType::Stop class="w-4 h-4 mr-2" />
                            "Stop"
                        </Button>
                        <Button variant=ButtonVariant::Secondary on:click=test_keyboard>
                            <Icon icon_type=IconType::Keyboard class="w-4 h-4 mr-2" />
                            "Test Keyboard"
                        </Button>
                        <Button variant=ButtonVariant::Ghost on:click=clear_dictation>
                            <Icon icon_type=IconType::X class="w-4 h-4 mr-2" />
                            "Clear"
                        </Button>
                    </div>
                    <div class="mt-4 p-4 bg-slate-50/50 rounded-xl border border-slate-200/60">
                        <p class="text-sm text-slate-600 leading-relaxed">
                            "Speak naturally and your words will be automatically typed into the focused application. Switch to your target app before starting dictation."
                        </p>
                    </div>
                            </Card>
                        </div>
                    </div>
                </main>
            </div>
        </div>
    }
}
