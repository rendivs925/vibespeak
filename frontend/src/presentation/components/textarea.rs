//! Reusable Textarea component

use leptos::*;

#[component]
pub fn Textarea(
    #[prop(optional, into)] value: String,
    #[prop(optional, into)] placeholder: String,
    #[prop(default = String::new())] class: String,
    #[prop(default = 3)] rows: usize,
) -> impl IntoView {
    let base_classes = "w-full px-4 py-3 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200 resize-vertical";

    let classes = format!("{} {}", base_classes, class);
    let rows_attr = rows.to_string();

    view! {
        <textarea
            class=classes
            prop:value=value
            placeholder=placeholder
            rows=rows_attr
        ></textarea>
    }
}
