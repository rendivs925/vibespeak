//! Navigation bar component

use crate::presentation::components::{Icon, IconType};
use leptos::*;
use leptos_router::*;

#[component]
pub fn NavBar(_active: &'static str) -> impl IntoView {
    view! {
        <nav class="w-full bg-white/80 backdrop-blur-md border-b border-slate-200/60 z-40">
            <div class="flex items-center justify-center gap-6 px-8 py-4">
                <A href="/" class="p-2 rounded-xl transition-all duration-200 hover:bg-slate-50">
                    <Icon icon_type=IconType::Home class="w-5 h-5 text-slate-500 hover:text-slate-700" />
                </A>
                <A href="/remote" class="p-2 rounded-xl transition-all duration-200 hover:bg-slate-50">
                    <Icon icon_type=IconType::Screen class="w-5 h-5 text-slate-500 hover:text-slate-700" />
                </A>
                <A href="/commands" class="p-2 rounded-xl transition-all duration-200 hover:bg-slate-50">
                    <Icon icon_type=IconType::Commands class="w-5 h-5 text-slate-500 hover:text-slate-700" />
                </A>
                <A href="/workflows" class="p-2 rounded-xl transition-all duration-200 hover:bg-slate-50">
                    <Icon icon_type=IconType::Workflows class="w-5 h-5 text-slate-500 hover:text-slate-700" />
                </A>
                <A href="/scripts" class="p-2 rounded-xl transition-all duration-200 hover:bg-slate-50">
                    <Icon icon_type=IconType::Scripts class="w-5 h-5 text-slate-500 hover:text-slate-700" />
                </A>
                <A href="/settings" class="p-2 rounded-xl transition-all duration-200 hover:bg-slate-50">
                    <Icon icon_type=IconType::Settings class="w-5 h-5 text-slate-500 hover:text-slate-700" />
                </A>
            </div>
        </nav>
    }
}
