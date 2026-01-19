//! Reusable DataTable component

use leptos::*;

#[component]
pub fn DataTable(#[prop(into)] headers: Vec<String>, children: Children) -> impl IntoView {
    view! {
        <div class="overflow-hidden bg-white rounded-2xl shadow-sm shadow-slate-100/50 border border-slate-200/60">
            <table class="w-full">
                <thead class="bg-slate-50/50 border-b border-slate-200">
                    <tr>
                        {headers.into_iter().map(|header| {
                            view! {
                                <th class="px-4 py-3 text-left text-xs font-semibold uppercase tracking-wide text-slate-600">
                                    {header}
                                </th>
                            }
                        }).collect::<Vec<_>>()}
                    </tr>
                </thead>
                <tbody class="text-gray-900 divide-y divide-slate-100">
                    {children()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
pub fn TableRow(children: Children) -> impl IntoView {
    view! {
        <tr class="hover:bg-slate-50/50 transition-colors duration-200">
            {children()}
        </tr>
    }
}

#[component]
pub fn TableCell(#[prop(optional, into)] class: String, children: Children) -> impl IntoView {
    let base_classes = "px-4 py-3 text-sm text-slate-700";
    let classes = format!("{} {}", base_classes, class);

    view! {
        <td class=classes>
            {children()}
        </td>
    }
}
