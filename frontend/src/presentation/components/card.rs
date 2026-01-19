//! Card component

use leptos::*;

#[component]
pub fn Card(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 p-6 mb-6">
            <h3 class="text-lg font-semibold text-gray-900 tracking-tight mb-4">{title}</h3>
            {children()}
        </div>
    }
}
