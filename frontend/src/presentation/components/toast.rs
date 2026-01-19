use leptos::*;
use std::time::Duration;

#[derive(Clone)]
pub struct ToastMessage {
    pub message: String,
    pub toast_type: ToastType,
    pub id: usize,
}

#[derive(Clone, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone)]
pub struct ToastContext {
    pub toasts: ReadSignal<Vec<ToastMessage>>,
    pub set_toasts: WriteSignal<Vec<ToastMessage>>,
}

impl ToastContext {
    pub fn new() -> Self {
        let (toasts, set_toasts) = create_signal(Vec::<ToastMessage>::new());

        // Auto-remove toasts after 5 seconds
        create_effect(move |_| {
            let toasts = toasts.get();
            for toast in toasts.iter() {
                let id = toast.id;
                let set_toasts = set_toasts;
                set_timeout(
                    move || {
                        set_toasts.update(|toasts| {
                            toasts.retain(|t| t.id != id);
                        });
                    },
                    Duration::from_secs(5),
                );
            }
        });

        Self { toasts, set_toasts }
    }

    pub fn show_success(&self, message: String) {
        self.set_toasts.update(|toasts| {
            toasts.push(ToastMessage {
                message,
                toast_type: ToastType::Success,
                id: js_sys::Math::random() as usize * 1000000,
            });
        });
    }

    pub fn show_error(&self, message: String) {
        self.set_toasts.update(|toasts| {
            toasts.push(ToastMessage {
                message,
                toast_type: ToastType::Error,
                id: js_sys::Math::random() as usize * 1000000,
            });
        });
    }

    pub fn show_warning(&self, message: String) {
        self.set_toasts.update(|toasts| {
            toasts.push(ToastMessage {
                message,
                toast_type: ToastType::Warning,
                id: js_sys::Math::random() as usize * 1000000,
            });
        });
    }

    pub fn show_info(&self, message: String) {
        self.set_toasts.update(|toasts| {
            toasts.push(ToastMessage {
                message,
                toast_type: ToastType::Info,
                id: js_sys::Math::random() as usize * 1000000,
            });
        });
    }
}

#[component]
pub fn ToastContainer() -> impl IntoView {
    let toast_context = expect_context::<ToastContext>();

    view! {
        <div class="fixed top-4 left-4 right-4 sm:left-auto sm:right-4 z-50 space-y-2 max-w-sm mx-auto sm:mx-0">
            <For
                each=move || toast_context.toasts.get()
                key=|toast| toast.id
                children=move |toast| {
                    let toast_id = toast.id;
                    view! {
                        <div class={
                            let toast_type = toast.toast_type.clone();
                            move || {
                                let base = "px-4 py-3 rounded-lg shadow-lg border transition-all duration-300 max-w-sm";
                                let colors = match toast_type {
                                    ToastType::Success => {
                                        "opacity-0 bg-emerald-50 border-emerald-200 text-emerald-800"
                                    }
                                    ToastType::Error => "bg-red-50 border-red-200 text-red-800",
                                    ToastType::Warning => {
                                        "bg-amber-50 border-amber-200 text-amber-800"
                                    }
                                    ToastType::Info => "bg-blue-50 border-blue-200 text-blue-800",
                                };
                                format!("{} {}", base, colors)
                            }
                        }>
                            <div class="flex items-start justify-between">
                                <div class="flex items-start">
                                    <div class="flex-shrink-0 mr-3">
                                        {move || match toast.toast_type {
                                            ToastType::Success => {
                                                view! {
                                                    <svg
                                                        class="w-5 h-5 text-emerald-400"
                                                        fill="currentColor"
                                                        viewBox="0 0 20 20"
                                                    >
                                                        <path
                                                            fill-rule="evenodd"
                                                            d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                                                            clip-rule="evenodd"
                                                        />
                                                    </svg>
                                                }
                                            }
                                            ToastType::Error => {
                                                view! {
                                                    <svg
                                                        class="w-5 h-5 text-red-400"
                                                        fill="currentColor"
                                                        viewBox="0 0 20 20"
                                                    >
                                                        <path
                                                            fill-rule="evenodd"
                                                            d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                                                            clip-rule="evenodd"
                                                        />
                                                    </svg>
                                                }
                                            }
                                            ToastType::Warning => {
                                                view! {
                                                    <svg
                                                        class="w-5 h-5 text-amber-400"
                                                        fill="currentColor"
                                                        viewBox="0 0 20 20"
                                                    >
                                                        <path
                                                            fill-rule="evenodd"
                                                            d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                                                            clip-rule="evenodd"
                                                        />
                                                    </svg>
                                                }
                                            }
                                            ToastType::Info => {
                                                view! {
                                                    <svg
                                                        class="w-5 h-5 text-blue-400"
                                                        fill="currentColor"
                                                        viewBox="0 0 20 20"
                                                    >
                                                        <path
                                                            fill-rule="evenodd"
                                                            d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                                                            clip-rule="evenodd"
                                                        />
                                                    </svg>
                                                }
                                            }
                                        }}
                                    </div>
                                    <div class="flex-1">
                                        <p class="text-sm font-medium">{&toast.message}</p>
                                    </div>
                                </div>
                                <button
                                    class="flex-shrink-0 ml-3 text-gray-400 hover:text-gray-600 transition-colors"
                                    on:click=move |_| {
                                        toast_context
                                            .set_toasts
                                            .update(|toasts| {
                                                toasts.retain(|t| t.id != toast_id);
                                            });
                                    }
                                >
                                    <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                        <path
                                            fill-rule="evenodd"
                                            d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                                            clip-rule="evenodd"
                                        />
                                    </svg>
                                </button>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}
