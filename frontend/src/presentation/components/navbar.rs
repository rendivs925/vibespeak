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
        <nav class="nav">
            <ul class="nav-tabs">
                <For
                    each=move || nav_items.clone()
                    key=|(id, _, _)| *id
                    children=move |(id, href, label)| {
                        let is_active = id == active;
                        let class = if is_active { "active" } else { "" };

                        view! {
                            <li>
                                <A href=href class=class>
                                    {label}
                                </A>
                            </li>
                        }
                    }
                />
            </ul>
        </nav>
    }
}
