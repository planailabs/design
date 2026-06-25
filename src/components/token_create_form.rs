use dioxus::prelude::*;
use dioxus_i18n::t;

use super::button::{Button, ButtonKind, ButtonSize};

/// What the consuming app receives when the shared create form is submitted.
/// App-specific fields (kind, scope) are owned by the caller via `children`
/// and read inside its `on_submit` handler.
#[derive(Clone, PartialEq)]
pub struct TokenCreateInput {
    pub label: String,
    /// Selected expiry as seconds-from-now; `None` means never expires.
    pub expires_in_secs: Option<i64>,
}

/// Compact, single-row token creation form — label + expiry selector +
/// optional extra fields (`children`, e.g. kind/scope selects) + submit.
///
/// The form owns the label and expiry signals and clears the label on submit;
/// the caller reads any `children` field signals inside `on_submit`. Mirrors
/// mac-mgmt's inline create row so every token surface looks the same.
#[component]
pub fn TokenCreateForm(
    on_submit: EventHandler<TokenCreateInput>,
    /// Submit button text; defaults to a localized "Create token".
    #[props(default, into)]
    submit_label: Option<String>,
    #[props(default)] submitting: bool,
    /// Extra inline fields rendered before the submit button (kind/scope).
    #[props(default)]
    children: Element,
) -> Element {
    let mut label = use_signal(String::new);
    let mut expiry = use_signal(|| "0".to_string()); // seconds; "0" = never

    rsx! {
        form {
            class: "flex flex-wrap gap-2 mb-4 items-end",
            onsubmit: move |evt| {
                evt.prevent_default();
                let secs: i64 = expiry.read().parse().unwrap_or(0);
                let input = TokenCreateInput {
                    label: label.read().clone(),
                    expires_in_secs: (secs > 0).then_some(secs),
                };
                label.set(String::new());
                on_submit.call(input);
            },
            input {
                class: "input flex-1 w-auto py-1 text-sm",
                r#type: "text",
                required: true,
                placeholder: t!("token-create-label"),
                value: "{label}",
                oninput: move |evt| label.set(evt.value()),
            }
            select {
                class: "input w-auto py-1 text-sm",
                value: "{expiry}",
                oninput: move |evt| expiry.set(evt.value()),
                option { value: "0", {t!("token-expiry-never")} }
                option { value: "3600", {t!("token-expiry-1h")} }
                option { value: "86400", {t!("token-expiry-24h")} }
                option { value: "604800", {t!("token-expiry-7d")} }
                option { value: "2592000", {t!("token-expiry-30d")} }
                option { value: "7776000", {t!("token-expiry-90d")} }
                option { value: "31536000", {t!("token-expiry-1y")} }
            }
            {children}
            Button { kind: ButtonKind::Submit, size: ButtonSize::Sm, disabled: submitting,
                {submit_label.unwrap_or_else(|| t!("token-create"))}
            }
        }
    }
}
