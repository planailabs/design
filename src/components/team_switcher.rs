//! Team switcher — the "which team am I working in" dropdown for the topbar,
//! plus the selection state pages read.
//!
//! The selection is a team id shared through context and remembered in
//! `localStorage['team']`. Apps load their own teams and map them to
//! [`TeamOption`]s; ids are opaque strings so any id type fits.
//!
//! ```ignore
//! // Layout (above the header and the router outlet):
//! let selection = use_team_selection_provider();
//! let current = resolve_team(selection().as_deref(), &teams); // per render
//! TeamSwitcher { teams }
//! // Anywhere below:
//! let selection = use_team_selection();
//! ```

use dioxus::prelude::*;
use dioxus_i18n::t;

const STORAGE_KEY: &str = "team";

/// A team as the switcher shows it.
#[derive(Clone, PartialEq, Debug)]
pub struct TeamOption {
    pub id: String,
    pub name: String,
    /// Secondary text after the name, e.g. the plan or your role.
    pub detail: Option<String>,
}

/// The selected team id (`None` until chosen or restored).
#[derive(Clone, Copy)]
pub struct TeamSelection(pub Signal<Option<String>>);

/// Creates the selection, restores the remembered team, and provides it to
/// everything below. Call once, in the layout.
pub fn use_team_selection_provider() -> Signal<Option<String>> {
    let mut selection = use_signal(|| None::<String>);
    use_context_provider(|| TeamSelection(selection));
    use_effect(move || {
        spawn(async move {
            let js = format!("try {{ return localStorage.getItem('{STORAGE_KEY}') || ''; }} catch(e) {{ return ''; }}");
            if let Ok(v) = document::eval(&js).await
                && let Some(id) = v.as_str().filter(|s| !s.is_empty())
            {
                selection.set(Some(id.to_string()));
            }
        });
    });
    selection
}

/// The selection provided by [`use_team_selection_provider`].
pub fn use_team_selection() -> Signal<Option<String>> {
    use_context::<TeamSelection>().0
}

/// Selects a team and remembers it.
pub fn select_team(mut selection: Signal<Option<String>>, id: String) {
    document::eval(&format!("try {{ localStorage.setItem('{STORAGE_KEY}', {id:?}); }} catch(e) {{}}"));
    selection.set(Some(id));
}

/// The selected team if it is still in `teams` (one may have been left or
/// deleted), else the first.
pub fn resolve_team<'a>(selected: Option<&str>, teams: &'a [TeamOption]) -> Option<&'a TeamOption> {
    selected.and_then(|id| teams.iter().find(|t| t.id == id)).or_else(|| teams.first())
}

/// Topbar dropdown over `teams`, bound to the provided selection.
#[component]
pub fn TeamSwitcher(teams: Vec<TeamOption>, #[props(default, into)] class: String) -> Element {
    let selection = use_team_selection();
    let current = resolve_team(selection().as_deref(), &teams).map(|t| t.id.clone()).unwrap_or_default();
    if teams.is_empty() {
        return rsx! {};
    }
    rsx! {
        div { class: "relative {class}",
            label { class: "sr-only", r#for: "team-switcher", {t!("team-switcher-label")} }
            select {
                id: "team-switcher",
                class: "input w-auto max-w-[16rem] py-1 pr-7 text-sm",
                title: t!("team-switcher-label"),
                value: "{current}",
                onchange: move |e| select_team(selection, e.value()),
                for team in teams.iter() {
                    option { key: "{team.id}", value: "{team.id}", selected: team.id == current,
                        match &team.detail {
                            Some(d) => rsx! { "{team.name} · {d}" },
                            None => rsx! { "{team.name}" },
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_first_team() {
        let t = |id: &str| TeamOption { id: id.into(), name: id.into(), detail: None };
        let teams = [t("a"), t("b")];
        assert_eq!(resolve_team(Some("b"), &teams).unwrap().id, "b");
        assert_eq!(resolve_team(Some("gone"), &teams).unwrap().id, "a");
        assert_eq!(resolve_team(None, &teams).unwrap().id, "a");
        assert!(resolve_team(Some("a"), &[]).is_none());
    }
}
