//! Header component

use leptos::*;

#[component]
pub fn Header(title: &'static str, subtitle: &'static str, children: Children) -> impl IntoView {
    view! {
        <header class="bg-gradient-to-r from-indigo-600 to-indigo-700 text-white py-8 px-8 shadow-sm shadow-slate-100/50">
            <div class="max-w-6xl mx-auto">
                <div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
                    <div class="space-y-2">
                        <h1 class="text-3xl font-semibold text-white tracking-tight">{title}</h1>
                        <p class="text-base text-indigo-100 leading-relaxed max-w-2xl">{subtitle}</p>
                    </div>
                    <div class="flex-shrink-0">
                        {children()}
                    </div>
                </div>
            </div>
        </header>
    }
}
