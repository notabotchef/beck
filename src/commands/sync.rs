//! `beck sync`: two personalities behind one subcommand.
//!
//! 1. Default path (`beck sync`, no `--from`): the Phase 1 behavior.
//!    Walk configured skill roots, rebuild the SQLite FTS5 index.
//!    Untouched by v0.2.
//!
//! 2. Reverse ingest (`beck sync --from <agent> [--write] [--force]`):
//!    new in Phase 6. Pulls skills from an agent's native directory
//!    into the canonical `~/beck/skills/` tree so beck takes over
//!    as the source of record. Dry-run by default. Conflict-aware.
//!
//! The split is intentional. The v0.1 sync surface is load-bearing for
//! existing users; we do not want to change its defaults. The `--from`
//! flag is the one knob that flips the mode.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

use beck::agents::adapter::Adapter;
use beck::agents::paths::skills_home;
use beck::agents::registry;
use beck::agents::skill::Skill;
use beck::db::Db;
use beck::error::{CliError, Result};
use beck::paths;
use beck::sync as core_sync;

pub async fn handle(
    _force: bool,
    json_out: bool,
    from: Option<String>,
    write: bool,
) -> Result<()> {
    if let Some(agent) = from {
        return handle_ingest(&agent, write, json_out).await;
    }

    // Default v0.1 path: rebuild the index from configured roots.
    let roots = paths::default_roots();
    if roots.is_empty() {
        return Err(CliError::Validation(
            "no skill roots found. Expected ~/.hermes/skills or ~/.claude/skills.".into(),
        ));
    }
    let db_path = paths::db_path()?;
    let db = Db::open(&db_path).map_err(CliError::Other)?;
    db.clear().map_err(CliError::Other)?;

    let mut total = 0usize;
    let mut per_root: Vec<(String, usize)> = Vec::new();
    for root in &roots {
        let n = core_sync::sync_root(&db, root).map_err(CliError::Other)?;
        per_root.push((root.display().to_string(), n));
        total += n;
    }

    if json_out {
        let payload = json!({
            "indexed": total,
            "db": db_path.display().to_string(),
            "roots": per_root.iter().map(|(p, n)| json!({"path": p, "indexed": n})).collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else {
        println!("indexed {total} skills into {}", db_path.display());
        for (path, n) in &per_root {
            println!("  {n:>4}  {path}");
        }
    }
    Ok(())
}

async fn handle_ingest(agent: &str, write: bool, json_out: bool) -> Result<()> {
    let adapter = registry::find_adapter(agent).ok_or_else(|| {
        CliError::Validation(format!(
            "unknown agent {agent:?}, known: {}",
            registry::known_agent_names()
        ))
    })?;

    if !adapter.detect() {
        return Err(CliError::Validation(format!(
            "agent {agent} not detected on this machine"
        )));
    }

    let skills_root = skills_home()?;
    let report = run_ingest(adapter.as_ref(), &skills_root, write, false)?;

    if json_out {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        print_ingest_report(&report);
    }

    if write && report.conflicts > 0 {
        return Err(CliError::Validation(
            "one or more conflicts, pass --force to overwrite canonical sources".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, Default, Serialize, PartialEq)]
pub struct IngestReport {
    pub agent: String,
    pub plans: Vec<IngestPlan>,
    pub created: u32,
    pub skipped: u32,
    pub conflicts: u32,
    pub written: bool,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum IngestPlan {
    Create { skill: String, target: PathBuf },
    Skip { skill: String, reason: String },
    Conflict { skill: String, target: PathBuf },
}

/// Core ingest logic. Takes the adapter to walk, the canonical skills
/// root, and the write/force flags. Never touches stdout.
pub fn run_ingest(
    adapter: &dyn Adapter,
    skills_root: &Path,
    write: bool,
    force: bool,
) -> Result<IngestReport> {
    let ingested = adapter.ingest()?;
    let mut report = IngestReport {
        agent: adapter.name().to_string(),
        written: write,
        ..Default::default()
    };

    for skill in ingested {
        let target_dir = skills_root.join(&skill.name);
        let target = target_dir.join("SKILL.md");

        let plan = classify(&skill, &target)?;
        report.plans.push(plan.clone());

        match plan {
            IngestPlan::Create { .. } => {
                if write {
                    fs::create_dir_all(&target_dir).map_err(|e| {
                        CliError::Validation(format!(
                            "mkdir -p {} failed: {e}",
                            target_dir.display()
                        ))
                    })?;
                    atomic_write(&target, &skill.source_path)?;
                }
                report.created += 1;
            }
            IngestPlan::Skip { .. } => {
                report.skipped += 1;
            }
            IngestPlan::Conflict { target, .. } => {
                if write && force {
                    fs::create_dir_all(&target_dir).map_err(|e| {
                        CliError::Validation(format!(
                            "mkdir -p {} failed: {e}",
                            target_dir.display()
                        ))
                    })?;
                    atomic_write(&target, &skill.source_path)?;
                    // Force counts as create, not conflict.
                    report.created += 1;
                    // Remove the conflict plan we pushed a second ago
                    // and replace it with Create so the report is
                    // consistent with what actually ran.
                    if let Some(last) = report.plans.last_mut() {
                        *last = IngestPlan::Create {
                            skill: skill.name.clone(),
                            target: target.clone(),
                        };
                    }
                } else {
                    report.conflicts += 1;
                }
            }
        }
    }

    Ok(report)
}

fn classify(skill: &Skill, target: &Path) -> Result<IngestPlan> {
    if !target.exists() {
        return Ok(IngestPlan::Create {
            skill: skill.name.clone(),
            target: target.to_path_buf(),
        });
    }

    // Compare byte-for-byte against the source we just loaded. If the
    // on-disk canonical source is identical, nothing to do.
    let existing = fs::read(target).map_err(|e| {
        CliError::Validation(format!(
            "cannot read existing {} for hash compare: {e}",
            target.display()
        ))
    })?;
    let source = fs::read(&skill.source_path).map_err(|e| {
        CliError::Validation(format!(
            "cannot read source {} for hash compare: {e}",
            skill.source_path.display()
        ))
    })?;

    if existing == source {
        Ok(IngestPlan::Skip {
            skill: skill.name.clone(),
            reason: "target already matches source byte-for-byte".into(),
        })
    } else {
        Ok(IngestPlan::Conflict {
            skill: skill.name.clone(),
            target: target.to_path_buf(),
        })
    }
}

fn atomic_write(target: &Path, source: &Path) -> Result<()> {
    let bytes = fs::read(source).map_err(|e| {
        CliError::Validation(format!(
            "failed to read source {}: {e}",
            source.display()
        ))
    })?;
    let tmp = {
        let mut os = target.as_os_str().to_os_string();
        os.push(".tmp");
        PathBuf::from(os)
    };
    fs::write(&tmp, &bytes).map_err(|e| {
        CliError::Validation(format!(
            "write to tmp {} failed: {e}",
            tmp.display()
        ))
    })?;
    fs::rename(&tmp, target).map_err(|e| {
        CliError::Validation(format!(
            "rename {} -> {} failed: {e}",
            tmp.display(),
            target.display()
        ))
    })?;
    Ok(())
}

fn print_ingest_report(report: &IngestReport) {
    println!(
        "agent: {}, dry_run: {}",
        report.agent,
        !report.written
    );
    println!(
        "  created: {}, skipped: {}, conflicts: {}",
        report.created, report.skipped, report.conflicts
    );
    for plan in &report.plans {
        match plan {
            IngestPlan::Create { skill, target } => {
                println!("  create {skill} -> {}", target.display())
            }
            IngestPlan::Skip { skill, reason } => {
                println!("  skip {skill}: {reason}")
            }
            IngestPlan::Conflict { skill, target } => {
                println!("  conflict {skill} at {}", target.display())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use beck::agents::adapter::InstallPlan;
    use beck::agents::manifest::{Entry, InstallMode};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn tempdir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "beck-ingest-tests-{name}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    struct StubAdapter {
        name: &'static str,
        skills: RefCell<Vec<Skill>>,
    }

    unsafe impl Send for StubAdapter {}
    unsafe impl Sync for StubAdapter {}

    impl Adapter for StubAdapter {
        fn name(&self) -> &'static str {
            self.name
        }
        fn detect(&self) -> bool {
            true
        }
        fn target_root(&self) -> Result<PathBuf> {
            Ok(PathBuf::from("/tmp/stub"))
        }
        fn plan(&self, _skill: &Skill) -> Result<InstallPlan> {
            unreachable!("plan not used")
        }
        fn install(&self, _plan: &InstallPlan) -> Result<Entry> {
            unreachable!("install not used")
        }
        fn uninstall(&self, _entry: &Entry) -> Result<()> {
            Ok(())
        }
        fn ingest(&self) -> Result<Vec<Skill>> {
            Ok(self.skills.borrow().clone())
        }
    }

    fn write_source_file(root: &Path, name: &str, body: &str) -> Skill {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("SKILL.md");
        fs::write(&path, body).unwrap();
        Skill::from_path(&path).unwrap()
    }

    #[test]
    fn classify_absent_is_create() {
        let src_root = tempdir("classify-absent-src");
        let dst_root = tempdir("classify-absent-dst");
        let skill = write_source_file(&src_root, "alpha", "body\n");

        let target = dst_root.join("alpha").join("SKILL.md");
        let plan = classify(&skill, &target).unwrap();
        match plan {
            IngestPlan::Create { skill: n, .. } => assert_eq!(n, "alpha"),
            other => panic!("expected Create, got {other:?}"),
        }
    }

    #[test]
    fn classify_matching_target_is_skip() {
        let src_root = tempdir("classify-skip-src");
        let dst_root = tempdir("classify-skip-dst");
        let skill = write_source_file(&src_root, "alpha", "matching body\n");

        let target_dir = dst_root.join("alpha");
        fs::create_dir_all(&target_dir).unwrap();
        let target = target_dir.join("SKILL.md");
        fs::write(&target, "matching body\n").unwrap();

        let plan = classify(&skill, &target).unwrap();
        assert!(matches!(plan, IngestPlan::Skip { .. }));
    }

    #[test]
    fn classify_drifted_target_is_conflict() {
        let src_root = tempdir("classify-conflict-src");
        let dst_root = tempdir("classify-conflict-dst");
        let skill = write_source_file(&src_root, "alpha", "source body\n");

        let target_dir = dst_root.join("alpha");
        fs::create_dir_all(&target_dir).unwrap();
        let target = target_dir.join("SKILL.md");
        fs::write(&target, "different body\n").unwrap();

        let plan = classify(&skill, &target).unwrap();
        assert!(matches!(plan, IngestPlan::Conflict { .. }));
    }

    #[test]
    fn run_ingest_dry_run_writes_nothing() {
        let src_root = tempdir("ingest-dry-src");
        let dst_root = tempdir("ingest-dry-dst");
        let skill = write_source_file(&src_root, "alpha", "body\n");

        let adapter = StubAdapter {
            name: "stub",
            skills: RefCell::new(vec![skill]),
        };
        let report = run_ingest(&adapter, &dst_root, false, false).unwrap();

        assert_eq!(report.agent, "stub");
        assert!(!report.written);
        assert_eq!(report.created, 1);
        assert!(!dst_root.join("alpha").join("SKILL.md").exists());
    }

    #[test]
    fn run_ingest_write_creates_file_atomically() {
        let src_root = tempdir("ingest-write-src");
        let dst_root = tempdir("ingest-write-dst");
        let skill = write_source_file(&src_root, "alpha", "body\n");

        let adapter = StubAdapter {
            name: "stub",
            skills: RefCell::new(vec![skill]),
        };
        let report = run_ingest(&adapter, &dst_root, true, false).unwrap();

        assert_eq!(report.created, 1);
        assert!(report.written);
        let target = dst_root.join("alpha").join("SKILL.md");
        assert!(target.exists());
        assert_eq!(fs::read_to_string(&target).unwrap(), "body\n");

        // No leftover .tmp file.
        let tmp = dst_root.join("alpha").join("SKILL.md.tmp");
        assert!(!tmp.exists());
    }

    #[test]
    fn run_ingest_conflict_without_force_is_recorded_not_written() {
        let src_root = tempdir("ingest-conflict-src");
        let dst_root = tempdir("ingest-conflict-dst");
        let skill = write_source_file(&src_root, "alpha", "source body\n");

        // Pre-existing canonical source with different contents.
        fs::create_dir_all(dst_root.join("alpha")).unwrap();
        let target = dst_root.join("alpha").join("SKILL.md");
        fs::write(&target, "pre-existing body\n").unwrap();

        let adapter = StubAdapter {
            name: "stub",
            skills: RefCell::new(vec![skill]),
        };
        let report = run_ingest(&adapter, &dst_root, true, false).unwrap();

        assert_eq!(report.conflicts, 1);
        assert_eq!(report.created, 0);
        // File on disk is unchanged.
        assert_eq!(
            fs::read_to_string(&target).unwrap(),
            "pre-existing body\n"
        );
    }

    #[test]
    fn run_ingest_conflict_with_force_overwrites() {
        let src_root = tempdir("ingest-force-src");
        let dst_root = tempdir("ingest-force-dst");
        let skill = write_source_file(&src_root, "alpha", "source body\n");

        fs::create_dir_all(dst_root.join("alpha")).unwrap();
        let target = dst_root.join("alpha").join("SKILL.md");
        fs::write(&target, "pre-existing body\n").unwrap();

        let adapter = StubAdapter {
            name: "stub",
            skills: RefCell::new(vec![skill]),
        };
        let report = run_ingest(&adapter, &dst_root, true, true).unwrap();

        assert_eq!(report.conflicts, 0);
        assert_eq!(report.created, 1);
        assert_eq!(fs::read_to_string(&target).unwrap(), "source body\n");
    }

    #[test]
    fn ingest_report_serializes_cleanly() {
        let report = IngestReport {
            agent: "claude-code".into(),
            plans: vec![IngestPlan::Create {
                skill: "alpha".into(),
                target: PathBuf::from("/tmp/alpha/SKILL.md"),
            }],
            created: 1,
            skipped: 0,
            conflicts: 0,
            written: true,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"agent\":\"claude-code\""));
        assert!(json.contains("\"kind\":\"create\""));
        assert!(json.contains("\"created\":1"));
    }
}
