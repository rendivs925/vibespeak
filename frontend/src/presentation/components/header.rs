//! Header component

use leptos::*;

#[component]
pub fn Header(
    title: &'static str,
    subtitle: &'static str,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <header class="bg-gradient-to-r from-indigo-600 to-indigo-700 text-white py-4 sm:py-6 px-4 sm:px-8">
            <div class="max-w-6xl mx-auto">
                <div class="flex items-center justify-between">
                    <div class="min-w-0 flex-1">
                        <h1 class="text-xl sm:text-2xl font-semibold text-white tracking-tight truncate">{title}</h1>
                        <p class="text-xs sm:text-sm text-indigo-100/80 mt-1 hidden sm:block">{subtitle}</p>
                    </div>
                     <div class="flex-shrink-0 ml-4">
                         {children.map(|c| c()).unwrap_or_else(|| Fragment::new(vec![])) }
                     </div>
                </div>
            </div>
        </header>
    }
}
