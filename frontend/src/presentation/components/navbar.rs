//! Navigation bar component

use leptos::*;
use leptos_router::*;

#[component]
pub fn NavBar(active: &'static str) -> impl IntoView {
    let nav_items = vec![
        ("dashboard", "/", "Dashboard"),
        ("remote", "/remote", "Remote Control"),
        ("commands", "/commands", "Voice Commands"),
        ("workflows", "/workflows", "Workflows"),
        ("scripts", "/scripts", "Scripts"),
        ("settings", "/settings", "Settings"),
    ];

    view! {
        <nav class="w-full bg-white/80 backdrop-blur-md border-b border-slate-200/60 z-40">
            <div class="flex items-center gap-8 px-8 py-3 text-sm">
                <For
                    each=move || nav_items.clone()
                    key=|(id, _, _)| *id
                    children=move |(id, href, label)| {
                        let is_active = id == active;
                        let base_classes = "text-slate-600 hover:text-slate-900 transition-colors duration-200 font-medium tracking-tight";
                        let active_classes = if is_active {
                            "text-indigo-700 font-semibold"
                        } else {
                            ""
                        };

                        view! {
                            <A
                                href=href
                                class=format!("{} {}", base_classes, active_classes)
                            >
                                {label}
                            </A>
                        }
                    }
                />
            </div>
        </nav>
    }
}
