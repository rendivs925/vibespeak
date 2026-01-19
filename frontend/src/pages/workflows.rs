//! Workflows page component

use crate::components::{Card, Header, NavBar, StatusBadge};
use leptos::*;

#[component]
pub fn Workflows() -> impl IntoView {
    let (status, _set_status) = create_signal("Ready".to_string());
    let (status_type, _set_status_type) = create_signal("info".to_string());

    view! {
        <div class="container">
            <Header title="Vibespeak" subtitle="Voice Automation System - Control your computer with your voice">
                <StatusBadge message=status status_type=status_type />
            </Header>

            <NavBar active="workflows" />

            <div class="content">
                <h2>"Workflows"</h2>

                <Card title="Automation Workflows">
                    <p>"Build complex automation sequences triggered by voice commands."</p>
                    <button class="btn">"Create Workflow"</button>
                    <div style="margin-top: 20px;">
                        <p>"No workflows configured yet."</p>
                    </div>
                </Card>
            </div>
        </div>
    }
}
