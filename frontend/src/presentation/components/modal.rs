//! Ultra-Premium Modal component

use crate::presentation::components::{Icon, IconType};
use leptos::*;

#[component]
pub fn Modal(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    title: &'static str,
    #[prop(optional)] subtitle: Option<&'static str>,
    #[prop(optional, into)] size: String,
    #[prop(optional)] icon: Option<IconType>,
    children: ChildrenFn,
) -> impl IntoView {
    let size_class = match size.as_str() {
        "sm" => "sm:max-w-md",
        "lg" => "sm:max-w-2xl",
        "xl" => "sm:max-w-3xl",
        "2xl" => "sm:max-w-4xl",
        "full" => "sm:max-w-6xl",
        _ => "sm:max-w-lg",
    };

    let close = on_close.clone();
    let close_on_backdrop = move |_| {
        close.call(());
    };

    let close_btn = on_close.clone();
    let close_on_button = move |_| {
        close_btn.call(());
    };

    view! {
        <Show when=move || is_open.get()>
            <div class="fixed inset-0 z-50 overflow-y-auto">
                // Backdrop
                <div
                    class="fixed inset-0 bg-slate-900/60 backdrop-blur-sm transition-opacity duration-300"
                    on:click=close_on_backdrop
                ></div>

                // Modal Container
                <div class="flex min-h-screen items-center justify-center p-4">
                    <div class=format!(
                        "relative w-full {} transform overflow-hidden rounded-2xl bg-white shadow-2xl shadow-slate-900/20 transition-all duration-300",
                        size_class
                    )>
                        // Header
                        <div class="flex items-start justify-between px-7 py-5 border-b border-slate-100 bg-gradient-to-r from-white to-slate-50/30">
                            <div class="flex items-center gap-4">
                                {icon.map(|i| view! {
                                    <div class="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500 to-indigo-600 flex items-center justify-center shadow-md shadow-indigo-200/50">
                                        <Icon icon_type=i class="w-5 h-5 text-white" />
                                    </div>
                                })}
                                <div>
                                    <h3 class="text-lg font-semibold text-gray-900 tracking-tight">{title}</h3>
                                    {subtitle.map(|s| view! {
                                        <p class="text-sm text-slate-500 mt-0.5">{s}</p>
                                    })}
                                </div>
                            </div>
                            <button
                                class="p-2 rounded-xl text-slate-400 hover:text-slate-600 hover:bg-slate-100 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-indigo-500/20"
                                on:click=close_on_button
                            >
                                <Icon icon_type=IconType::X class="w-5 h-5" />
                            </button>
                        </div>

                        // Body
                        <div class="px-7 py-6 bg-gradient-to-b from-white to-slate-50/30">
                            {children()}
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

/// Modal footer component for actions
#[component]
pub fn ModalFooter(children: Children) -> impl IntoView {
    view! {
        <div class="flex items-center justify-end gap-3 pt-6 mt-6 border-t border-slate-100">
            {children()}
        </div>
    }
}
