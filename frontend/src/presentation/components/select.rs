//! Reusable Select component

use leptos::*;

#[component]
pub fn Select(
    #[prop(optional, into)] value: String,
    #[prop(default = String::new())] class: String,
    children: Children,
) -> impl IntoView {
    let base_classes = "w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200 appearance-none bg-[url('data:image/svg+xml;charset=utf-8,%3Csvg xmlns=%27http://www.w3.org/2000/svg%27 fill=%27none%27 viewBox=%270 0 20 20%27%3E%3Cpath stroke=%27%236b7280%27 stroke-linecap=%27round%27 stroke-linejoin=%27round%27 stroke-width=%271.5%27 d=%27m6 8 4 4 4-4%27/%3E%3C/svg%3E')] bg-[length:1.5em_1.5em] bg-[right_0.5rem_center] bg-no-repeat pr-10";

    let classes = format!("{} {}", base_classes, class);

    view! {
        <select
            class=classes
            prop:value=value
        >
            {children()}
        </select>
    }
}
