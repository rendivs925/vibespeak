//! Reusable Modal component

use leptos::*;

#[component]
pub fn Modal(
    #[prop(into)] is_open: Signal<bool>,
    #[prop(into)] title: String,
    #[prop(optional, into)] size: String,
    children: Children,
) -> impl IntoView {
    let size_class = match size.as_str() {
        "sm" => "sm:max-w-md",
        "lg" => "sm:max-w-lg",
        "xl" => "sm:max-w-xl",
        "2xl" => "sm:max-w-2xl",
        _ => "sm:max-w-lg",
    };

    view! {
        <div class=move || if is_open.get() { "fixed inset-0 z-50 overflow-y-auto" } else { "hidden" }>
            <div class="flex min-h-screen items-center justify-center px-4 pt-4 pb-20 text-center sm:block sm:p-0">
                <div class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"></div>
                <div class=format!("inline-block transform overflow-hidden rounded-2xl bg-white text-left align-bottom shadow-2xl shadow-slate-200/50 transition-all sm:my-8 sm:w-full {} sm:align-middle", size_class)>
                    <div class="bg-white px-7 py-5 border-b border-slate-200">
                        <h3 class="text-lg font-semibold text-gray-900 tracking-tight">{title}</h3>
                    </div>
                    <div class="px-7 py-6">
                        {children()}
                    </div>
                </div>
            </div>
        </div>
    }
}
