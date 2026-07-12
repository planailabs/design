//! Shared scaffolding for list pages.
//!
//! `DataTable` owns the toolbar + Card + table + thead + tbody markup.
//! Pages own data fetching, filtering, and sorting; they pass headers
//! and rows in as Element slots.
//!
//! ```ignore
//! let (start, shown) = page_window(*page.read(), limit_val, filtered_count);
//! DataTable {
//!     search, limit, page, total, filtered: filtered_count, shown,
//!     headers: rsx! {
//!         SortableTh { label: t!("name"), sort_key: "name".to_string(), sort }
//!     },
//!     body: rsx! {
//!         for row in filtered.read().iter().skip(start).take(limit_val) {
//!             tr { key: "{row.id}",
//!                 Td { Link { to: ..., class: "link", "{row.name}" } }
//!             }
//!         }
//!     },
//! }
//! ```

use dioxus::prelude::*;
use dioxus_i18n::t;

use super::button::{Button, ButtonSize, ButtonVariant};
use super::card::Card;

/// Sort state shared between a list page's signal and `SortableTh`.
/// (column key, ascending) — empty key means unsorted.
pub type SortState = (String, bool);

/// Clamped `(start, shown)` window for the current page. Pages slice
/// their filtered rows with `.skip(start).take(limit)`. Clamping keeps
/// the window valid when filtering shrinks the row set below the
/// stored page index (the signal itself is only reset on search/limit
/// changes, never during render).
pub fn page_window(page: usize, limit: usize, filtered: usize) -> (usize, usize) {
    let limit = limit.max(1);
    let last = filtered.saturating_sub(1) / limit;
    let start = page.min(last) * limit;
    (start, filtered.saturating_sub(start).min(limit))
}

// ── DataTable shell ────────────────────────────────────────────────

#[component]
pub fn DataTable(
    search: Signal<String>,
    limit: Signal<usize>,
    page: Signal<usize>,
    total: usize,
    filtered: usize,
    shown: usize,
    headers: Element,
    body: Element,
) -> Element {
    rsx! {
        TableToolbar { search, limit, page, total, filtered, shown }
        Card {
            // Tables hold their natural width via column content; on
            // narrow viewports we let the user scroll horizontally
            // rather than wrapping cells. Without this wrap the Card's
            // `overflow-hidden` would clip the right edge silently.
            div { class: "overflow-x-auto",
                table { class: "table",
                    thead { class: "thead",
                        tr { {headers} }
                    }
                    tbody { class: "tbody",
                        {body}
                    }
                }
            }
        }
    }
}

// ── Header cells ───────────────────────────────────────────────────

/// Plain (non-sortable) header cell.
#[component]
pub fn Th(children: Element) -> Element {
    rsx! { th { class: "th", {children} } }
}

#[component]
pub fn SortableTh(label: String, sort_key: String, sort: Signal<SortState>) -> Element {
    let cur = sort.read().clone();
    let arrow = if cur.0 == sort_key {
        if cur.1 { " \u{2191}" } else { " \u{2193}" }
    } else {
        ""
    };
    let key_click = sort_key.clone();
    rsx! {
        th { class: "th-sortable",
            onclick: move |_| {
                let (k, a) = sort.read().clone();
                if k == key_click {
                    sort.set((k, !a));
                } else {
                    sort.set((key_click.clone(), true));
                }
            },
            "{label}{arrow}"
        }
    }
}

// ── Toolbar (search + page-size + counts + pager) ──────────────────

#[component]
pub fn TableToolbar(
    search: Signal<String>,
    limit: Signal<usize>,
    page: Signal<usize>,
    total: usize,
    filtered: usize,
    shown: usize,
) -> Element {
    let limit_val = (*limit.read()).max(1);
    let pages = filtered.div_ceil(limit_val).max(1);
    // Same clamping as `page_window` — the signal may point past the
    // last page after the row set shrinks.
    let cur = (*page.read()).min(pages - 1);
    rsx! {
        // Wraps to two rows below `sm` so the search box doesn't shove
        // the row counter and page-size dropdown off the right edge.
        // Search input stretches to fill on phones (`w-full sm:w-64`).
        div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between mb-3 gap-3",
            div { class: "relative w-full sm:w-auto",
                input {
                    class: "input py-1.5 w-full sm:w-64 pl-8",
                    r#type: "text",
                    placeholder: t!("search-placeholder"),
                    value: "{search}",
                    oninput: move |evt| {
                        search.set(evt.value());
                        page.set(0);
                    },
                }
                svg {
                    class: "absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-fg-faint",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    view_box: "0 0 24 24",
                    circle { cx: "11", cy: "11", r: "8" }
                    line { x1: "21", y1: "21", x2: "16.65", y2: "16.65" }
                }
            }
            div { class: "flex items-center gap-3 help",
                if total != filtered {
                    span { {t!("table-showing-filtered", shown: shown, filtered: filtered, total: total)} }
                } else {
                    span { {t!("table-showing", shown: shown, total: total)} }
                }
                select {
                    class: "input w-auto py-1.5",
                    value: "{limit}",
                    onchange: move |evt| {
                        if let Ok(n) = evt.value().parse::<usize>() {
                            limit.set(n);
                            page.set(0);
                        }
                    },
                    option { value: "20", {t!("table-per-page-20")} }
                    option { value: "50", {t!("table-per-page-50")} }
                    option { value: "100", {t!("table-per-page-100")} }
                }
                if pages > 1 {
                    div { class: "flex items-center gap-2",
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Xs,
                            disabled: cur == 0,
                            onclick: move |_| page.set(cur.saturating_sub(1)),
                            {t!("table-prev")}
                        }
                        span { class: "whitespace-nowrap", {t!("table-page", page: cur + 1, pages: pages)} }
                        Button {
                            variant: ButtonVariant::Ghost,
                            size: ButtonSize::Xs,
                            disabled: cur + 1 >= pages,
                            onclick: move |_| page.set(cur + 1),
                            {t!("table-next")}
                        }
                    }
                }
            }
        }
    }
}

// ── Cell helpers ───────────────────────────────────────────────────

/// Standard body cell. `<td class="td">{children}</td>`.
#[component]
pub fn Td(#[props(default, into)] class: String, children: Element) -> Element {
    rsx! { td { class: "td {class}", {children} } }
}

/// Muted (secondary) body cell.
#[component]
pub fn TdMuted(#[props(default, into)] class: String, children: Element) -> Element {
    rsx! { td { class: "td-muted {class}", {children} } }
}

/// Monospace body cell (small text).
#[component]
pub fn TdMono(#[props(default, into)] class: String, children: Element) -> Element {
    rsx! { td { class: "td-mono {class}", {children} } }
}

/// Inline placeholder for empty values. Renders the localized "—".
#[component]
pub fn Dash() -> Element {
    rsx! { span { class: "text-fg-faint italic", {t!("dash")} } }
}
