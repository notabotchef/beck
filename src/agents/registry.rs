//! The adapter registry: the single place that knows which adapters beck
//! ships in this version.
//!
//! v0.2 ships exactly one adapter: `ClaudeCodeAdapter`. Cursor is
//! deferred to v0.3 (see `.rune/plan-beck-link-spec.md` §0). Phase 4
//! (`beck link`) and Phase 5 (`beck check`) call into this registry to
//! iterate the set.
//!
//! Keeping the set in code (not config) is deliberate: the binary size
//! budget rules out dynamic dispatch over user-supplied plugins, and the
//! CLI surface `--agent <name>` must be a closed set for tab-completion
//! and docs.

use crate::agents::adapter::Adapter;
use crate::agents::claude_code::ClaudeCodeAdapter;

/// Return every adapter beck ships in this build, in the order they will
/// be iterated by `beck link` and `beck check`.
pub fn all_adapters() -> Vec<Box<dyn Adapter>> {
    vec![Box::new(ClaudeCodeAdapter::new())]
}

/// Look up an adapter by its `name()`, matching the value users pass to
/// `beck link --agent <name>`. Returns `None` for unknown names; callers
/// turn that into `CliError::Validation` with a list of known names.
pub fn find_adapter(name: &str) -> Option<Box<dyn Adapter>> {
    all_adapters().into_iter().find(|a| a.name() == name)
}

/// Comma-separated list of every shipping adapter name. Used by the CLI
/// to print "known agents: claude-code" when the user passes an
/// unrecognized `--agent`.
pub fn known_agent_names() -> String {
    all_adapters()
        .iter()
        .map(|a| a.name())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_adapters_ships_claude_code() {
        let adapters = all_adapters();
        assert_eq!(adapters.len(), 1);
        assert_eq!(adapters[0].name(), "claude-code");
    }

    #[test]
    fn find_adapter_returns_claude_code() {
        let adapter = find_adapter("claude-code").expect("present");
        assert_eq!(adapter.name(), "claude-code");
    }

    #[test]
    fn find_adapter_returns_none_for_unknown() {
        assert!(find_adapter("cursor").is_none());
        assert!(find_adapter("").is_none());
        assert!(find_adapter("CLAUDE-CODE").is_none());
    }

    #[test]
    fn known_agent_names_is_comma_joined() {
        let names = known_agent_names();
        assert_eq!(names, "claude-code");
    }
}
