//! Header component

use leptos::*;

#[component]
pub fn Header(title: &'static str, subtitle: &'static str, children: Children) -> impl IntoView {
    view! {
        <header class="bg-gradient-to-r from-indigo-600 to-indigo-700 text-white py-12 px-6 text-center shadow-sm shadow-slate-100/50">
            <div class="max-w-4xl mx-auto">
                <h1 class="text-3xl font-semibold text-white mb-3 tracking-tight">{title}</h1>
                <p class="text-base text-indigo-100 leading-relaxed max-w-2xl mx-auto">{subtitle}</p>
                <div class="mt-6">
                    {children()}
                </div>
            </div>
        </header>
    }
}
