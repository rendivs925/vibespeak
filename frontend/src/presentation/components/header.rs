//! Header component

use leptos::*;

#[component]
pub fn Header(title: &'static str, subtitle: &'static str, children: Children) -> impl IntoView {
    view! {
        <header class="bg-gradient-to-r from-indigo-600 to-indigo-700 text-white py-6 px-8">
            <div class="max-w-6xl mx-auto">
                <div class="flex items-center justify-between">
                    <div>
                        <h1 class="text-2xl font-semibold text-white tracking-tight">{title}</h1>
                        <p class="text-sm text-indigo-100/80 mt-1">{subtitle}</p>
                    </div>
                    <div class="flex-shrink-0">
                        {children()}
                    </div>
                </div>
            </div>
        </header>
    }
}
