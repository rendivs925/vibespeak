//! Reusable Button component

use leptos::*;

#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

#[derive(Clone, PartialEq)]
pub enum ButtonSize {
    Small,
    Medium,
    Large,
}

#[component]
pub fn Button(
    #[prop(into)] variant: ButtonVariant,
    #[prop(into)] size: ButtonSize,
    #[prop(optional)] disabled: bool,
    #[prop(default = String::new())] class: String,
    children: Children,
) -> impl IntoView {
    let base_classes = "font-medium rounded-xl transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed";

    let variant_classes = match variant {
        ButtonVariant::Primary => "bg-gradient-to-r from-indigo-600 to-indigo-700 text-white shadow-md shadow-indigo-200/50 hover:shadow-lg hover:shadow-indigo-300/50 hover:from-indigo-700 hover:to-indigo-800 focus:ring-indigo-500",
        ButtonVariant::Secondary => "bg-white text-gray-700 border border-gray-300 hover:bg-gray-50 hover:border-gray-400 focus:ring-gray-500",
        ButtonVariant::Danger => "bg-red-600 text-white border border-red-600 hover:bg-red-700 hover:border-red-700 shadow-md shadow-red-200/50 hover:shadow-lg hover:shadow-red-300/50 focus:ring-red-500",
        ButtonVariant::Ghost => "text-gray-700 hover:bg-gray-100 focus:ring-gray-500",
    };

    let size_classes = match size {
        ButtonSize::Small => "px-3 py-1.5 text-xs",
        ButtonSize::Medium => "px-4 py-2 text-sm",
        ButtonSize::Large => "px-6 py-3 text-base",
    };

    let classes = format!(
        "{} {} {} {}",
        base_classes, variant_classes, size_classes, class
    );

    if disabled {
        view! {
            <button
                class=classes
                disabled
            >
                {children()}
            </button>
        }
    } else {
        view! {
            <button class=classes>
                {children()}
            </button>
        }
    }
}
