use dioxus::prelude::*;
use dioxus_i18n::t;

use super::alert::{Alert, AlertVariant};
use super::button::{Button, ButtonSize, ButtonVariant};

/// One-time secret reveal: a success-tinted box showing a freshly generated
/// secret (API token, deploy token, secret URL) for the user to copy. The
/// value is meant to be shown once, right after creation.
///
/// `label` is the heading — callers pass their own localized string (e.g.
/// "New admin token", "Deploy token"). `children` carries optional usage
/// notes (curl snippets, hints). The copy button writes the value to the
/// clipboard; the value block is also `select-all` for manual selection.
#[component]
pub fn TokenReveal(
    /// The secret to reveal (raw token or secret URL).
    value: String,
    /// Heading shown above the value; omitted if `None`.
    #[props(default, into)]
    label: Option<String>,
    /// Show the "copy now — it won't be shown again" reminder. Default on.
    #[props(default = true)]
    hint: bool,
    /// Optional extra content below the value (usage examples, notes).
    #[props(default)]
    children: Element,
) -> Element {
    let mut copied = use_signal(|| false);
    let val_for_copy = value.clone();
    rsx! {
        Alert { variant: AlertVariant::Success, class: "mb-4",
            if let Some(l) = label {
                p { class: "text-sm font-medium mb-1", "{l}" }
            }
            div { class: "flex items-stretch gap-2",
                code { class: "block flex-1 break-all select-all text-xs bg-success-soft p-2 rounded", "{value}" }
                Button {
                    variant: ButtonVariant::Secondary,
                    size: ButtonSize::Sm,
                    class: "shrink-0",
                    onclick: move |_| {
                        let v = val_for_copy.clone();
                        copied.set(true);
                        spawn(async move {
                            let _ = document::eval(&format!(
                                "navigator.clipboard.writeText({v:?}).catch(function(){{}});"
                            )).await;
                        });
                    },
                    if *copied.read() { {t!("token-reveal-copied")} } else { {t!("token-reveal-copy")} }
                }
            }
            if hint {
                p { class: "text-xs text-fg-muted mt-1", {t!("token-reveal-hint")} }
            }
            {children}
        }
    }
}
