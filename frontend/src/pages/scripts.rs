//! Scripts page component

use leptos::*;
use crate::components::{Header, NavBar, StatusBadge, Card};

#[component]
pub fn Scripts() -> impl IntoView {
    let (status, _set_status) = create_signal("Ready".to_string());
    let (status_type, _set_status_type) = create_signal("info".to_string());

    view! {
        <div class="container">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar active="scripts" />

            <div class="content">
                <h2>"Scripts"</h2>

                <Card title="Script Management">
                    <p>"Execute bash, Python, and JavaScript scripts via voice commands."</p>
                    <button class="btn">"Add Script"</button>
                    <div style="margin-top: 20px;">
                        <p>"No scripts configured yet."</p>
                    </div>
                </Card>
            </div>
        </div>
    }
}
