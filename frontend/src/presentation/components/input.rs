//! Reusable Input component

use leptos::*;

#[derive(Clone, PartialEq)]
pub enum InputType {
    Text,
    Email,
    Password,
    Number,
    Tel,
    Url,
    Search,
}

#[component]
pub fn Input(
    #[prop(into)] input_type: InputType,
    #[prop(optional, into)] value: String,
    #[prop(optional, into)] placeholder: String,
    #[prop(default = String::new())] class: String,
) -> impl IntoView {
    let input_type_str = match input_type {
        InputType::Text => "text",
        InputType::Email => "email",
        InputType::Password => "password",
        InputType::Number => "number",
        InputType::Tel => "tel",
        InputType::Url => "url",
        InputType::Search => "search",
    };

    let base_classes = "w-full px-4 py-2.5 text-sm text-gray-900 bg-white border border-slate-200/80 rounded-xl shadow-sm shadow-slate-100/30 placeholder:text-gray-400 hover:border-slate-300/80 focus:outline-none focus:ring-2 focus:ring-indigo-500/20 focus:border-indigo-500 transition-all duration-200";

    let classes = format!("{} {}", base_classes, class);

    view! {
        <input
            type=input_type_str
            class=classes
            prop:value=value
            placeholder=placeholder
        />
    }
}
