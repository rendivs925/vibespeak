//! Header component

use leptos::*;

#[component]
pub fn Header(
    title: &'static str,
    subtitle: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="header" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; text-align: center;">
            <h1>{title}</h1>
            <p>{subtitle}</p>
            {children()}
        </div>
    }
}
