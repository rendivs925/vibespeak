//! Reusable Badge component

use leptos::*;

#[derive(Clone, PartialEq)]
pub enum BadgeVariant {
    Success,
    Error,
    Warning,
    Info,
    Neutral,
}

#[component]
pub fn Badge(#[prop(into)] variant: BadgeVariant, #[prop(into)] text: String) -> impl IntoView {
    let (bg_class, text_class) = match variant {
        BadgeVariant::Success => ("bg-emerald-100", "text-emerald-800"),
        BadgeVariant::Error => ("bg-red-100", "text-red-800"),
        BadgeVariant::Warning => ("bg-amber-100", "text-amber-800"),
        BadgeVariant::Info => ("bg-blue-100", "text-blue-800"),
        BadgeVariant::Neutral => ("bg-gray-100", "text-gray-800"),
    };

    view! {
        <span class=format!("inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {} {}", bg_class, text_class)>
            {text}
        </span>
    }
}
