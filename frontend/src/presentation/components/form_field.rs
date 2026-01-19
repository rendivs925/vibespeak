//! Reusable FormField component

use leptos::*;

#[component]
pub fn FormField(
    #[prop(into)] label: String,
    #[prop(optional, into)] help_text: String,
    #[prop(optional)] required: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="space-y-2">
            <label class="block text-sm font-medium text-gray-700">
                {label}
                {move || if required {
                    view! { <span class="text-red-500 ml-1">*</span> }
                } else {
                    view! { <span></span> }
                }}
            </label>
            {children()}
            {move || (!help_text.is_empty()).then(|| view! {
                <p class="mt-1.5 text-xs text-gray-500">{help_text.clone()}</p>
            })}
        </div>
    }
}
