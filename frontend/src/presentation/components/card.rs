//! Card component

use leptos::*;

#[component]
pub fn Card(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60 overflow-hidden">
            <div class="px-7 py-5 border-b border-slate-100">
                <h3 class="text-lg font-semibold text-gray-900 tracking-tight">{title}</h3>
            </div>
            <div class="px-7 pt-6 pb-8 bg-gradient-to-b from-white to-slate-50/30">
                {children()}
            </div>
        </div>
    }
}
