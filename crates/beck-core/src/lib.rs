//! beck-core: indexing and query for SKILL.md files.
//!
//! Phase 0 scope (eval gate): frontmatter parser, in-memory FTS5 index,
//! and BM25 ranked query. Persistent storage, sync triggers, and the
//! full CLI land in later phases per HANDOFF.md.

pub mod frontmatter;
pub mod db;
pub mod sync;
pub mod query;

pub use frontmatter::Frontmatter;
pub use db::Db;
pub use query::{search, Match};
