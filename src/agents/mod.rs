//! Agent-facing layer: skills home, manifest, and (in later phases) adapters
//! that translate a canonical `~/beck/skills/<name>/SKILL.md` into a file
//! another agent (Claude Code, Cursor, ...) actually reads.
//!
//! Phase 1 of `beck-link` ships the foundation: path resolution + manifest
//! load/save. Adapters arrive in Phase 2.

pub mod manifest;
pub mod paths;
