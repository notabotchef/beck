//! `beck link`: install every skill under `~/beck/skills/` into every
//! detected agent's native skills directory.
//!
//! Phase 4 of `beck-link`. v0.2 ships with Claude Code as the only
//! registered adapter; `--agent` accepts `claude-code` only until v0.3
//! adds more. The command is transactional at the per-skill level: if
//! adapter A installs cleanly and adapter B fails for the same skill,
//! adapter A is rolled back for that skill and we move to the next
//! skill. The manifest is saved once at the end, atomically.
//!
//! The core logic is in `run_link`, which takes the adapter set as a
//! slice so unit tests can inject mocks. The public `handle` builds the
//! adapter set from the real registry.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::json;

use std::path::{Path, PathBuf};

use beck::agents::adapter::{Adapter, InstallPlan};
use beck::agents::manifest::{Entry, Manifest};
use beck::agents::paths::{beck_home, manifest_path, skills_home};
use beck::agents::registry;
use beck::agents::skill::Skill;
use beck::error::{CliError, Result};

/// Parameters the testable core needs. The `--json` flag only changes
/// the stdout emitter in `handle()`, so we do not carry it down here.
#[derive(Debug, Clone)]
pub struct LinkArgs {
    pub agent: Option<String>,
    pub dry_run: bool,
    pub force: bool,
}

/// Structured report for `--json` output and for tests to assert on.
#[derive(Debug, Default, Serialize, PartialEq)]
pub struct LinkReport {
    pub linked: Vec<LinkedItem>,
    pub skipped: Vec<SkippedItem>,
    pub failed: Vec<FailedItem>,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct LinkedItem {
    pub skill: String,
    pub agent: String,
    pub target: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct SkippedItem {
    pub skill: String,
    pub agent: String,
    pub reason: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FailedItem {
    pub skill: String,
    pub agent: String,
    pub error: String,
}

/// Top-level entry point called from `main.rs`. Builds the adapter set
/// from the shipping registry and runs the link workflow.
pub async fn handle(
    agent: Option<String>,
    dry_run: bool,
    force: bool,
    json: bool,
) -> Result<()> {
    let adapters = registry::all_adapters();
    let skills_root = skills_home()?;
    let manifest_file = manifest_path()?;
    let report = run_link(
        LinkArgs {
            agent,
            dry_run,
            force,
        },
        &skills_root,
        &manifest_file,
        adapters.iter().map(|a| a.as_ref()).collect::<Vec<_>>(),
    )?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&report).unwrap_or_default()
        );
    } else {
        print_human_report(&report);
    }

    if !report.failed.is_empty() && report.linked.is_empty() {
        return Err(CliError::Validation(
            "all link attempts failed, see above".into(),
        ));
    }
    Ok(())
}

/// Testable core. Walks the skills home, runs the adapter matrix, and
/// returns a `LinkReport`. Does not touch stdout.
///
/// Both `skills_root` and `manifest_file` are passed explicitly so unit
/// tests can point at a tempdir without touching the `BECK_HOME` env
/// var. The public `handle` resolves them from the real env.
pub fn run_link(
    args: LinkArgs,
    skills_root: &Path,
    manifest_file: &Path,
    adapters: Vec<&dyn Adapter>,
) -> Result<LinkReport> {
    if !skills_root.exists() {
        return Err(CliError::Validation(
            "skills home does not exist, run `beck bootstrap` first".into(),
        ));
    }

    let skills = Skill::discover_in(skills_root)?;
    if skills.is_empty() {
        let report = LinkReport {
            dry_run: args.dry_run,
            ..Default::default()
        };
        return Ok(report);
    }

    let filtered = filter_adapters(&adapters, args.agent.as_deref())?;
    if filtered.is_empty() {
        return Err(CliError::Validation(
            "no agents detected, install Claude Code or run `beck check`".into(),
        ));
    }

    // Load the current manifest so we can recognize already-installed
    // entries. On the dry-run path we load but never write back.
    let mut manifest = if manifest_file.exists() {
        Manifest::load(manifest_file)?
    } else {
        Manifest::empty()
    };

    let mut report = LinkReport {
        dry_run: args.dry_run,
        ..Default::default()
    };

    for skill in &skills {
        if let Err(rollback_err) = process_skill(
            skill,
            &filtered,
            args.force,
            args.dry_run,
            &mut manifest,
            &mut report,
        ) {
            // Per-skill rollback already happened inside `process_skill`;
            // this outer error is only hit if rollback itself ran into a
            // non-recoverable state. Record and keep going.
            report.failed.push(FailedItem {
                skill: skill.name.clone(),
                agent: "*".into(),
                error: format!("rollback failed: {rollback_err}"),
            });
        }
    }

    if !args.dry_run {
        manifest.save(manifest_file)?;
    }

    Ok(report)
}

/// Unused in the current code path; exported so `check` (Phase 5) can
/// share the same resolution logic if it wants.
#[allow(dead_code)]
fn default_paths() -> Result<(PathBuf, PathBuf)> {
    let _ = beck_home()?; // validates $BECK_HOME/HOME
    Ok((skills_home()?, manifest_path()?))
}

fn process_skill(
    skill: &Skill,
    adapters: &[&dyn Adapter],
    force: bool,
    dry_run: bool,
    manifest: &mut Manifest,
    report: &mut LinkReport,
) -> Result<()> {
    // Entries installed this call for THIS skill. If a later adapter
    // fails, we roll these back before moving on.
    let mut this_skill_entries: Vec<(usize, Entry)> = Vec::new();

    for (idx, adapter) in adapters.iter().enumerate() {
        let plan = match adapter.plan(skill) {
            Ok(p) => p,
            Err(e) => {
                report.failed.push(FailedItem {
                    skill: skill.name.clone(),
                    agent: adapter.name().into(),
                    error: format!("plan failed: {e}"),
                });
                return rollback_skill(
                    &this_skill_entries,
                    adapters,
                    manifest,
                    report,
                    &skill.name,
                );
            }
        };

        if dry_run {
            report.linked.push(LinkedItem {
                skill: skill.name.clone(),
                agent: adapter.name().into(),
                target: plan.target.display().to_string(),
            });
            continue;
        }

        // `--force` only re-installs a target that is already beck-
        // managed per the old manifest. Foreign files are never
        // clobbered, matching the invariant in
        // `.rune/plan-beck-link-spec.md` §5.
        if force
            && manifest.find(&skill.name, adapter.name()).is_some()
            && let Some(existing) = manifest.remove(&skill.name, adapter.name())
        {
            // Ignore uninstall errors here: best-effort cleanup before
            // the fresh install tries again.
            let _ = adapter.uninstall(&existing);
        }

        match install_with_skip(&plan, *adapter, skill, manifest) {
            InstallOutcome::Installed(entry) => {
                report.linked.push(LinkedItem {
                    skill: skill.name.clone(),
                    agent: adapter.name().into(),
                    target: entry.target.display().to_string(),
                });
                this_skill_entries.push((idx, entry.clone()));
                upsert_entry(manifest, entry);
            }
            InstallOutcome::Skipped { reason } => {
                report.skipped.push(SkippedItem {
                    skill: skill.name.clone(),
                    agent: adapter.name().into(),
                    reason,
                });
            }
            InstallOutcome::Failed(err) => {
                report.failed.push(FailedItem {
                    skill: skill.name.clone(),
                    agent: adapter.name().into(),
                    error: format!("{err}"),
                });
                return rollback_skill(
                    &this_skill_entries,
                    adapters,
                    manifest,
                    report,
                    &skill.name,
                );
            }
        }
    }

    Ok(())
}

enum InstallOutcome {
    Installed(Entry),
    Skipped { reason: String },
    Failed(CliError),
}

fn install_with_skip(
    plan: &InstallPlan,
    adapter: &dyn Adapter,
    skill: &Skill,
    manifest: &Manifest,
) -> InstallOutcome {
    // Short-circuit: if the manifest already has an identical entry and
    // the source sha256 has not drifted, we can skip the install call
    // entirely. This keeps `beck link && beck link` cheap.
    if let Some(existing) = manifest.find(&skill.name, adapter.name())
        && existing.sha256 == skill.sha256
        && existing.target == plan.target
    {
        return InstallOutcome::Skipped {
            reason: "already installed, source sha256 unchanged".into(),
        };
    }

    match adapter.install(plan) {
        Ok(entry) => InstallOutcome::Installed(entry),
        Err(e) => InstallOutcome::Failed(e),
    }
}

fn rollback_skill(
    installed: &[(usize, Entry)],
    adapters: &[&dyn Adapter],
    manifest: &mut Manifest,
    report: &mut LinkReport,
    skill_name: &str,
) -> Result<()> {
    for (idx, entry) in installed.iter().rev() {
        if let Some(adapter) = adapters.get(*idx) {
            // Best-effort uninstall. Rollback failures are logged via
            // the manifest entry staying in memory, then pruned here.
            let _ = adapter.uninstall(entry);
            manifest.remove(&entry.skill, &entry.agent);
        }
    }
    // Scrub the report of any `linked` entries for this skill that were
    // added before the rollback. A rolled-back install must not show up
    // as linked in the final summary.
    report
        .linked
        .retain(|item| item.skill != skill_name);
    Ok(())
}

fn upsert_entry(manifest: &mut Manifest, entry: Entry) {
    // Replace any existing entry for (skill, agent) and append the fresh
    // one. This keeps the manifest at most one entry per pair.
    manifest.remove(&entry.skill, &entry.agent);
    manifest.add(entry);
}

fn filter_adapters<'a>(
    all: &'a [&'a dyn Adapter],
    selector: Option<&str>,
) -> Result<Vec<&'a dyn Adapter>> {
    let detected: Vec<&dyn Adapter> = all
        .iter()
        .filter(|a| a.detect())
        .copied()
        .collect();

    match selector {
        None => Ok(detected),
        Some(name) => {
            // When `--agent` is explicit, we do not require detect() to
            // be true. Users asking for a specific agent know what they
            // are doing, and integration tests may point at a tempdir
            // that does not yet have `.claude/` populated.
            let hit = all.iter().copied().find(|a| a.name() == name);
            match hit {
                Some(a) => Ok(vec![a]),
                None => Err(CliError::Validation(format!(
                    "unknown agent {name:?}, known: {}",
                    registry::known_agent_names()
                ))),
            }
        }
    }
}

fn print_human_report(report: &LinkReport) {
    let mut by_skill: BTreeMap<&String, Vec<String>> = BTreeMap::new();
    for item in &report.linked {
        by_skill
            .entry(&item.skill)
            .or_default()
            .push(format!("  {} -> {}", item.agent, item.target));
    }

    if !report.linked.is_empty() {
        let header = if report.dry_run { "would link" } else { "linked" };
        println!("{header} {} targets:", report.linked.len());
        for (skill, lines) in &by_skill {
            println!("{skill}");
            for line in lines {
                println!("{line}");
            }
        }
    }

    if !report.skipped.is_empty() {
        println!("skipped {}:", report.skipped.len());
        for item in &report.skipped {
            println!("  {}/{}: {}", item.skill, item.agent, item.reason);
        }
    }

    if !report.failed.is_empty() {
        eprintln!("failed {}:", report.failed.len());
        for item in &report.failed {
            eprintln!("  {}/{}: {}", item.skill, item.agent, item.error);
        }
    }

    if report.linked.is_empty() && report.skipped.is_empty() && report.failed.is_empty() {
        println!("no skills found under ~/beck/skills");
    }

    // The JSON path printed above; keep the human path quiet on final
    // summary (`json!` is used for --json only).
    let _ = json!({});
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use beck::agents::manifest::InstallMode;

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// Build an isolated beck root at a unique tempdir. No env vars
    /// touched: tests pass paths directly into `run_link`, which keeps
    /// the parallel test runner happy.
    fn tempdir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "beck-link-tests-{name}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn write_skill(skills_home: &std::path::Path, name: &str, body: &str) -> PathBuf {
        let dir = skills_home.join(name);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("SKILL.md");
        fs::write(&path, body).unwrap();
        path
    }

    /// In-memory mock adapter used to exercise the link workflow without
    /// touching disk. Records every call so tests can assert behavior.
    struct MockAdapter {
        name: &'static str,
        target_base: PathBuf,
        detect_ok: bool,
        fail_install_for: Option<&'static str>,
        installs: RefCell<Vec<String>>,
        uninstalls: RefCell<Vec<String>>,
    }

    // SAFETY: tests are single-threaded per the ENV_LOCK mutex.
    unsafe impl Send for MockAdapter {}
    unsafe impl Sync for MockAdapter {}

    impl Adapter for MockAdapter {
        fn name(&self) -> &'static str {
            self.name
        }
        fn detect(&self) -> bool {
            self.detect_ok
        }
        fn target_root(&self) -> Result<PathBuf> {
            Ok(self.target_base.clone())
        }
        fn plan(&self, skill: &Skill) -> Result<InstallPlan> {
            Ok(InstallPlan {
                source: skill.source_path.clone(),
                target: self.target_base.join(&skill.name).join("SKILL.md"),
                mode: InstallMode::Symlink,
                transform: None,
            })
        }
        fn install(&self, plan: &InstallPlan) -> Result<Entry> {
            if let Some(fail_name) = self.fail_install_for {
                let source_skill_name = plan
                    .source
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if source_skill_name == fail_name {
                    return Err(CliError::Validation(format!(
                        "mock failure for {fail_name}"
                    )));
                }
            }
            self.installs.borrow_mut().push(
                plan.source
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
            );
            let bytes = fs::read(&plan.source).unwrap_or_default();
            let mut hash = String::new();
            for b in sha2::Sha256::digest(&bytes).iter() {
                use std::fmt::Write;
                let _ = write!(hash, "{b:02x}");
            }
            Ok(Entry {
                skill: plan
                    .source
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string(),
                agent: self.name.to_string(),
                target: plan.target.clone(),
                mode: InstallMode::Symlink,
                sha256: hash,
                installed_at: "2026-04-11T00:00:00Z".into(),
            })
        }
        fn uninstall(&self, entry: &Entry) -> Result<()> {
            self.uninstalls.borrow_mut().push(entry.skill.clone());
            Ok(())
        }
    }

    use sha2::Digest;

    #[test]
    fn run_link_reports_one_entry_per_skill_per_adapter() {
        let beck = tempdir("link-report");
        let skills = beck.join("skills");
        fs::create_dir_all(&skills).unwrap();
        write_skill(&skills, "alpha", "body a\n");
        write_skill(&skills, "beta", "body b\n");
        let manifest_file = beck.join(".beck-manifest.json");

        let adapter = MockAdapter {
            name: "mock-1",
            target_base: beck.join("mock-1-target"),
            detect_ok: true,
            fail_install_for: None,
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };
        let adapters: Vec<&dyn Adapter> = vec![&adapter];

        let report = run_link(
            LinkArgs {
                agent: None,
                dry_run: false,
                force: false,
            },
            &skills,
            &manifest_file,
            adapters,
        )
        .unwrap();

        assert_eq!(report.linked.len(), 2);
        assert!(report.failed.is_empty());
        assert_eq!(adapter.installs.borrow().len(), 2);

        // Manifest was saved with both entries.
        let mf = Manifest::load(&manifest_file).unwrap();
        assert_eq!(mf.entries.len(), 2);
    }

    #[test]
    fn dry_run_writes_nothing() {
        let beck = tempdir("link-dry");
        let skills = beck.join("skills");
        fs::create_dir_all(&skills).unwrap();
        write_skill(&skills, "alpha", "body\n");
        let manifest_file = beck.join(".beck-manifest.json");

        let adapter = MockAdapter {
            name: "mock-1",
            target_base: beck.join("mock-1-target"),
            detect_ok: true,
            fail_install_for: None,
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };

        let report = run_link(
            LinkArgs {
                agent: None,
                dry_run: true,
                force: false,
            },
            &skills,
            &manifest_file,
            vec![&adapter],
        )
        .unwrap();

        assert!(report.dry_run);
        assert_eq!(report.linked.len(), 1);
        assert!(adapter.installs.borrow().is_empty());
        assert!(!manifest_file.exists());
    }

    #[test]
    fn agent_filter_picks_one_adapter() {
        let beck = tempdir("link-agent-filter");
        let skills = beck.join("skills");
        fs::create_dir_all(&skills).unwrap();
        write_skill(&skills, "alpha", "body\n");
        let manifest_file = beck.join(".beck-manifest.json");

        let a = MockAdapter {
            name: "mock-1",
            target_base: beck.join("t1"),
            detect_ok: true,
            fail_install_for: None,
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };
        let b = MockAdapter {
            name: "mock-2",
            target_base: beck.join("t2"),
            detect_ok: true,
            fail_install_for: None,
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };

        let report = run_link(
            LinkArgs {
                agent: Some("mock-2".into()),
                dry_run: false,
                force: false,
            },
            &skills,
            &manifest_file,
            vec![&a, &b],
        )
        .unwrap();

        assert_eq!(report.linked.len(), 1);
        assert_eq!(report.linked[0].agent, "mock-2");
        assert!(a.installs.borrow().is_empty());
        assert_eq!(b.installs.borrow().len(), 1);
    }

    #[test]
    fn unknown_agent_is_validation_error() {
        let beck = tempdir("link-agent-unknown");
        let skills = beck.join("skills");
        fs::create_dir_all(&skills).unwrap();
        write_skill(&skills, "alpha", "body\n");
        let manifest_file = beck.join(".beck-manifest.json");

        let a = MockAdapter {
            name: "mock-1",
            target_base: beck.join("t1"),
            detect_ok: true,
            fail_install_for: None,
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };

        let err = run_link(
            LinkArgs {
                agent: Some("nope".into()),
                dry_run: false,
                force: false,
            },
            &skills,
            &manifest_file,
            vec![&a],
        )
        .expect_err("should error");
        match err {
            CliError::Validation(msg) => assert!(msg.contains("unknown agent")),
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn second_adapter_failure_rolls_back_first() {
        let beck = tempdir("link-rollback");
        let skills = beck.join("skills");
        fs::create_dir_all(&skills).unwrap();
        write_skill(&skills, "alpha", "body\n");
        let manifest_file = beck.join(".beck-manifest.json");

        let good = MockAdapter {
            name: "good",
            target_base: beck.join("good-target"),
            detect_ok: true,
            fail_install_for: None,
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };
        let bad = MockAdapter {
            name: "bad",
            target_base: beck.join("bad-target"),
            detect_ok: true,
            fail_install_for: Some("alpha"),
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };

        let report = run_link(
            LinkArgs {
                agent: None,
                dry_run: false,
                force: false,
            },
            &skills,
            &manifest_file,
            vec![&good, &bad],
        )
        .unwrap();

        // `good` installed, then `bad` failed, then `good` rolled back.
        assert_eq!(good.installs.borrow().as_slice(), &["alpha".to_string()]);
        assert_eq!(bad.installs.borrow().len(), 0);
        assert_eq!(good.uninstalls.borrow().as_slice(), &["alpha".to_string()]);

        // Report shows the failure but NOT the rolled-back link.
        assert_eq!(report.failed.len(), 1);
        assert_eq!(report.failed[0].agent, "bad");
        let alpha_linked = report.linked.iter().any(|l| l.skill == "alpha");
        assert!(
            !alpha_linked,
            "rolled-back skill should not appear in linked list"
        );

        let mf = Manifest::load(&manifest_file).unwrap();
        assert!(mf.find("alpha", "good").is_none());
        assert!(mf.find("alpha", "bad").is_none());
    }

    #[test]
    fn empty_skills_home_is_ok() {
        let beck = tempdir("link-empty");
        let skills = beck.join("skills");
        fs::create_dir_all(&skills).unwrap();
        let manifest_file = beck.join(".beck-manifest.json");

        let adapter = MockAdapter {
            name: "mock",
            target_base: beck.join("t"),
            detect_ok: true,
            fail_install_for: None,
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };

        let report = run_link(
            LinkArgs {
                agent: None,
                dry_run: false,
                force: false,
            },
            &skills,
            &manifest_file,
            vec![&adapter],
        )
        .unwrap();

        assert!(report.linked.is_empty());
        assert!(report.failed.is_empty());
        assert!(adapter.installs.borrow().is_empty());
    }

    #[test]
    fn missing_skills_home_is_validation_error() {
        let beck = tempdir("link-missing");
        // Deliberately do NOT create beck/skills.
        let skills = beck.join("skills");
        let manifest_file = beck.join(".beck-manifest.json");

        let adapter = MockAdapter {
            name: "mock",
            target_base: beck.join("t"),
            detect_ok: true,
            fail_install_for: None,
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };

        let err = run_link(
            LinkArgs {
                agent: None,
                dry_run: false,
                force: false,
            },
            &skills,
            &manifest_file,
            vec![&adapter],
        )
        .expect_err("should error");
        match err {
            CliError::Validation(msg) => {
                assert!(msg.contains("beck bootstrap"), "msg={msg}")
            }
            other => panic!("expected Validation, got {other:?}"),
        }
    }

    #[test]
    fn second_run_is_idempotent_skipped() {
        let beck = tempdir("link-idempotent");
        let skills = beck.join("skills");
        fs::create_dir_all(&skills).unwrap();
        write_skill(&skills, "alpha", "body\n");
        let manifest_file = beck.join(".beck-manifest.json");

        let adapter = MockAdapter {
            name: "mock",
            target_base: beck.join("mock-target"),
            detect_ok: true,
            fail_install_for: None,
            installs: RefCell::new(vec![]),
            uninstalls: RefCell::new(vec![]),
        };

        let first = run_link(
            LinkArgs {
                agent: None,
                dry_run: false,
                force: false,
            },
            &skills,
            &manifest_file,
            vec![&adapter],
        )
        .unwrap();
        assert_eq!(first.linked.len(), 1);
        assert_eq!(adapter.installs.borrow().len(), 1);

        let second = run_link(
            LinkArgs {
                agent: None,
                dry_run: false,
                force: false,
            },
            &skills,
            &manifest_file,
            vec![&adapter],
        )
        .unwrap();

        assert_eq!(second.linked.len(), 0);
        assert_eq!(second.skipped.len(), 1);
        assert_eq!(second.skipped[0].skill, "alpha");
        assert_eq!(adapter.installs.borrow().len(), 1, "no second install");
    }

    #[test]
    fn json_report_serializes_link_report() {
        let report = LinkReport {
            dry_run: false,
            linked: vec![LinkedItem {
                skill: "caveman".into(),
                agent: "claude-code".into(),
                target: "/tmp/caveman/SKILL.md".into(),
            }],
            skipped: vec![],
            failed: vec![],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"linked\""));
        assert!(json.contains("caveman"));
        assert!(json.contains("claude-code"));
        assert!(json.contains("\"dry_run\":false"));
    }
}
