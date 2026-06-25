use dioxus::prelude::*;
use dioxus_i18n::t;

use super::badge::{Badge, BadgeVariant};
use super::button::{Button, ButtonSize, ButtonVariant};
use super::card::Card;
use super::data_table::{Td, TdMuted, Th};

/// A row in the shared [`TokenTable`]. This is a presentational view-model —
/// the consuming app maps its own token type into it. `id` is passed back to
/// the `on_revoke` handler.
#[derive(Clone, PartialEq)]
pub struct TokenRow {
    pub id: String,
    pub label: String,
    /// Token kind (shown only when the table's `show_kind` is set).
    pub kind: Option<String>,
    /// Scope, e.g. "admin", "org: Acme", "cluster: edge-1" (shown only when
    /// the table's `show_scope` is set).
    pub scope: Option<String>,
    pub revoked: bool,
    /// Past its expiry. Shown as a distinct status and suppresses revoke.
    pub expired: bool,
    pub created: String,
    /// Expiry (shown only when `show_expires` is set); `None` renders "never".
    pub expires: Option<String>,
}

/// Shared token listing — the single source of truth for the token table look
/// across apps. Columns: Label, [Kind], Status, Created, [Expires], [Actions].
///
/// `Kind`/`Expires` columns appear only when their flag is set; the `Actions`
/// (revoke) column appears only when `on_revoke` is provided — non-revoked
/// rows then get a revoke button that calls back with the row `id`.
#[component]
pub fn TokenTable(
    rows: Vec<TokenRow>,
    #[props(default)] show_kind: bool,
    #[props(default)] show_scope: bool,
    #[props(default)] show_expires: bool,
    #[props(default)] on_revoke: Option<EventHandler<String>>,
) -> Element {
    let cols = 3
        + show_kind as usize
        + show_scope as usize
        + show_expires as usize
        + on_revoke.is_some() as usize;
    rsx! {
        Card {
            div { class: "overflow-x-auto",
                table { class: "table",
                    thead { class: "thead",
                        tr {
                            Th { {t!("token-col-label")} }
                            if show_kind { Th { {t!("token-col-kind")} } }
                            if show_scope { Th { {t!("token-col-scope")} } }
                            Th { {t!("token-col-status")} }
                            Th { {t!("token-col-created")} }
                            if show_expires { Th { {t!("token-col-expires")} } }
                            if on_revoke.is_some() { Th { {t!("token-col-actions")} } }
                        }
                    }
                    tbody { class: "tbody",
                        if rows.is_empty() {
                            tr { td { class: "td text-center text-fg-muted", colspan: "{cols}", {t!("token-empty")} } }
                        }
                        for row in rows.iter() {
                            tr { key: "{row.id}",
                                Td { "{row.label}" }
                                if show_kind {
                                    Td {
                                        if let Some(k) = &row.kind {
                                            Badge { variant: BadgeVariant::Info, "{k}" }
                                        }
                                    }
                                }
                                if show_scope {
                                    TdMuted {
                                        {row.scope.clone().unwrap_or_else(|| t!("token-scope-global"))}
                                    }
                                }
                                Td {
                                    if row.revoked {
                                        Badge { variant: BadgeVariant::Danger, {t!("token-status-revoked")} }
                                    } else if row.expired {
                                        Badge { variant: BadgeVariant::Warn, {t!("token-status-expired")} }
                                    } else {
                                        Badge { variant: BadgeVariant::Success, {t!("token-status-active")} }
                                    }
                                }
                                TdMuted { "{row.created}" }
                                if show_expires {
                                    TdMuted { {row.expires.clone().unwrap_or_else(|| t!("token-expires-never"))} }
                                }
                                if let Some(handler) = on_revoke {
                                    Td {
                                        if !row.revoked && !row.expired {
                                            Button {
                                                variant: ButtonVariant::Danger,
                                                size: ButtonSize::Xs,
                                                onclick: {
                                                    let id = row.id.clone();
                                                    move |_| handler.call(id.clone())
                                                },
                                                {t!("token-revoke")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
