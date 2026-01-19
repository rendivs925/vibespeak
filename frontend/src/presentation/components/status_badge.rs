use leptos::*;

#[component]
pub fn StatusBadge(message: ReadSignal<String>, status_type: ReadSignal<String>) -> impl IntoView {
    view! {
        <span class=move || {
            let stype = status_type.get();
            let base = "text-sm font-medium";
            let color = match stype.as_str() {
                "success" | "ready" => "text-emerald-300",
                "error" => "text-red-300",
                "warning" | "busy" => "text-amber-300",
                _ => "text-white/70",
            };
            format!("{} {}", base, color)
        }>{move || message.get()}</span>
    }
}
