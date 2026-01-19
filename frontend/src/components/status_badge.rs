//! Status badge component

use leptos::*;

#[component]
pub fn StatusBadge(
    message: ReadSignal<String>,
    status_type: ReadSignal<String>,
) -> impl IntoView {
    view! {
        <div
            class="status-badge"
            style=move || {
                let stype = status_type.get();
                let (bg, color, border) = match stype.as_str() {
                    "success" => ("#d4edda", "#155724", "#c3e6cb"),
                    "error" => ("#f8d7da", "#721c24", "#f5c6cb"),
                    "warning" => ("#fff3cd", "#856404", "#ffeeba"),
                    _ => ("#d1ecf1", "#0c5460", "#bee5eb"),
                };
                format!(
                    "padding: 10px; margin: 10px 0; border-radius: 4px; background: {}; color: {}; border: 1px solid {};",
                    bg, color, border
                )
            }
        >
            {move || message.get()}
        </div>
    }
}
