//! Status badge component

use leptos::*;

#[component]
pub fn StatusBadge(message: ReadSignal<String>, status_type: ReadSignal<String>) -> impl IntoView {
    view! {
        <div
            class=move || {
                let stype = status_type.get();
                let base_classes = "px-4 py-3 rounded-xl text-sm font-medium shadow-sm";
                let color_classes = match stype.as_str() {
                    "success" => "bg-emerald-50 border border-emerald-200 text-emerald-700 shadow-emerald-100/50",
                    "error" => "bg-red-50 border border-red-200 text-red-700 shadow-red-100/50",
                    "warning" => "bg-amber-50 border border-amber-200 text-amber-600 shadow-amber-100/50",
                    _ => "bg-blue-50 border border-blue-200 text-blue-700 shadow-blue-100/50",
                };
                format!("{} {}", base_classes, color_classes)
            }
        >
            {move || message.get()}
        </div>
    }
}
