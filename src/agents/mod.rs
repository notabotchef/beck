//! Agent-facing layer: skills home, manifest, and adapters that translate
//! a canonical `~/beck/skills/<name>/SKILL.md` into a file another agent
//! (Claude Code today, Cursor in v0.3+) actually reads.
//!
//! Phase 1 shipped paths + manifest. Phase 2 adds the `Skill` loader, the
//! `Adapter` trait, `ClaudeCodeAdapter`, and the registry that owns the
//! shipping set of adapters.

pub mod adapter;
pub mod claude_code;
pub mod manifest;
pub mod paths;
pub mod registry;
pub mod skill;
