//! The `Adapter` trait: the single abstraction every agent-specific
//! installer implements.
//!
//! A canonical SKILL.md at `<skills_home>/<name>/SKILL.md` is translated
//! into a file another agent actually reads. The Claude Code adapter ships
//! in v0.2; Cursor and friends are deferred to v0.3 pending research on
//! their user-global install points.
//!
//! Contract is locked in `.rune/plan-beck-link-spec.md` §1. Do not add
//! methods to the trait without bumping the spec, because every new
//! adapter will inherit them.

use std::path::PathBuf;

use crate::agents::manifest::{Entry, InstallMode};
use crate::agents::skill::Skill;
use crate::error::Result;

/// The plan an adapter produces for a single skill, before it touches
/// disk. Phase 4 (`beck link`) prints the plan in `--dry-run` mode.
///
/// `transform` is a pure function from a `Skill` to the bytes written to
/// `target`. `None` means "symlink the source file byte-for-byte"; that is
/// the Claude Code case.
#[derive(Debug)]
pub struct InstallPlan {
    /// Source file under `<skills_home>/<name>/SKILL.md`.
    pub source: PathBuf,
    /// Destination the adapter writes on `install()`.
    pub target: PathBuf,
    /// Symlink or Copy. Symlink is the default for adapters that do not
    /// need a format transform.
    pub mode: InstallMode,
    /// Pure transform applied to the `Skill` to produce the bytes written
    /// at `target` when `mode == Copy`. `None` means no transform.
    pub transform: Option<fn(&Skill) -> String>,
}

/// Every agent-specific installer implements this trait.
///
/// Lifetime contract:
/// 1. `plan(&skill)` is a pure read of `skill`. It must not touch disk
///    outside `target_root()` lookups.
/// 2. `install(&plan)` may create directories and the single file or
///    symlink at `plan.target`. It must not write anywhere else.
/// 3. `uninstall(&entry)` removes exactly the file `entry.target` and
///    must refuse if the file on disk is no longer beck-managed.
/// 4. Adapters are stateless and safe to share across threads. The
///    `Send + Sync` bound is load-bearing for the registry that owns
///    `Box<dyn Adapter>`.
pub trait Adapter: Send + Sync {
    /// Stable identifier written into `Entry::agent`. Must match the name
    /// Phase 4 accepts in `beck link --agent <name>`.
    fn name(&self) -> &'static str;

    /// True if the agent appears to be installed on the current machine.
    /// For Claude Code this means `~/.claude/` exists.
    fn detect(&self) -> bool;

    /// Root directory the adapter installs into, e.g.
    /// `~/.claude/skills/`. Returns `CliError::Validation` if the home
    /// directory cannot be resolved.
    fn target_root(&self) -> Result<PathBuf>;

    /// Compute the `InstallPlan` for a single skill. Pure: no disk writes.
    fn plan(&self, skill: &Skill) -> Result<InstallPlan>;

    /// Execute a plan. Returns the manifest `Entry` the caller should
    /// append. Idempotent: running twice on an already-beck-managed
    /// target returns the same entry without erroring.
    fn install(&self, plan: &InstallPlan) -> Result<Entry>;

    /// Remove a previously installed file. Must verify the file on disk
    /// is still beck-managed before touching it (no clobbering
    /// user-authored files).
    fn uninstall(&self, entry: &Entry) -> Result<()>;

    /// Phase 6 hook: walk the agent's native skills dir and return any
    /// skills that are NOT already symlinks back into `~/beck/skills/`.
    /// Default is empty for adapters that have not wired ingest yet.
    fn ingest(&self) -> Result<Vec<Skill>> {
        Ok(Vec::new())
    }
}
