//! `beck unlink`: remove beck-managed files an adapter installed.
//!
//! Manifest-driven. Only entries listed in `<beck_home>/.beck-manifest.json`
//! are candidates for removal. Foreign files at the adapter's target
//! directory are never touched (that is the exact point of storing a
//! manifest in the first place).
//!
//! Filters:
//! - no flags → refuses and prints a hint, to prevent accidental
//!   "uninstall everything" when the user meant to scope it.
//! - `--skill NAME` → every entry with that `skill` across all agents.
//! - `--agent NAME` → every entry with that `agent` across all skills.
//! - `--all` → literally every entry in the manifest.
//! - `--skill` + `--agent` → the intersection.
//!
//! Save behavior mirrors `beck link`: mutate the manifest in memory,
//! save once at the end.

use serde::Serialize;

use std::path::Path;

use beck::agents::adapter::Adapter;
use beck::agents::manifest::{Entry, Manifest};
use beck::agents::paths::manifest_path;
use beck::agents::registry;
use beck::error::{CliError, Result};

/// Parameters the testable core needs. The `--json` flag only changes
/// the stdout emitter in `handle()`, so we do not carry it down here.
#[derive(Debug, Clone)]
pub struct UnlinkArgs {
    pub skill: Option<String>,
    pub agent: Option<String>,
    pub all: bool,
}

#[derive(Debug, Default, Serialize, PartialEq)]
pub struct UnlinkReport {
    pub removed: Vec<RemovedItem>,
    pub failed: Vec<FailedItem>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RemovedItem {
    pub skill: String,
    pub agent: String,
    pub target: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FailedItem {
    pub skill: String,
    pub agent: String,
    pub error: String,
}

pub async fn handle(
    skill: Option<String>,
    agent: Option<String>,
    all: bool,
    json: bool,
) -> Result<()> {
    let adapters = registry::all_adapters();
    let manifest_file = manifest_path()?;
    let report = run_unlink(
        UnlinkArgs { skill, agent, all },
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

    if !report.failed.is_empty() && report.removed.is_empty() {
        return Err(CliError::Validation(
            "all unlink attempts failed, see above".into(),
        ));
    }
    Ok(())
}

pub fn run_unlink(
    args: UnlinkArgs,
    manifest_file: &Path,
    adapters: Vec<&dyn Adapter>,
) -> Result<UnlinkReport> {
    if args.skill.is_none() && args.agent.is_none() && !args.all {
        return Err(CliError::Validation(
            "refusing to unlink without a scope, pass --skill, --agent, or --all".into(),
        ));
    }

    if !manifest_file.exists() {
        // No manifest, nothing to remove. This is not an error: the
        // user's beck home may never have been bootstrapped on this
        // machine.
        return Ok(UnlinkReport::default());
    }

    let mut manifest = Manifest::load(manifest_file)?;

    // Snapshot the entries to remove BEFORE we mutate the manifest.
    // Matching is inclusive: every provided filter must hold.
    let to_remove: Vec<Entry> = manifest
        .entries
        .iter()
        .filter(|e| {
            let skill_ok = args.skill.as_deref().is_none_or(|s| e.skill == s);
            let agent_ok = args.agent.as_deref().is_none_or(|a| e.agent == a);
            skill_ok && agent_ok
        })
        .cloned()
        .collect();

    let mut report = UnlinkReport::default();

    for entry in &to_remove {
        let adapter = adapters.iter().copied().find(|a| a.name() == entry.agent);
        let Some(adapter) = adapter else {
            // No adapter registered for this manifest entry. Drop the
            // entry anyway (the user removed or renamed an agent). Log
            // it under `removed` with an explanatory target.
            manifest.remove(&entry.skill, &entry.agent);
            report.removed.push(RemovedItem {
                skill: entry.skill.clone(),
                agent: entry.agent.clone(),
                target: format!(
                    "{} (no adapter registered, manifest entry pruned)",
                    entry.target.display()
                ),
            });
            continue;
        };

        match adapter.uninstall(entry) {
            Ok(()) => {
                manifest.remove(&entry.skill, &entry.agent);
                report.removed.push(RemovedItem {
                    skill: entry.skill.clone(),
                    agent: entry.agent.clone(),
                    target: entry.target.display().to_string(),
                });
            }
            Err(e) => {
                report.failed.push(FailedItem {
                    skill: entry.skill.clone(),
                    agent: entry.agent.clone(),
                    error: format!("{e}"),
                });
            }
        }
    }

    manifest.save(manifest_file)?;
    Ok(report)
}

fn print_human_report(report: &UnlinkReport) {
    if !report.removed.is_empty() {
        println!("unlinked {}:", report.removed.len());
        for item in &report.removed {
            println!("  {}/{} -> {}", item.skill, item.agent, item.target);
        }
    }
    if !report.failed.is_empty() {
        eprintln!("failed {}:", report.failed.len());
        for item in &report.failed {
            eprintln!("  {}/{}: {}", item.skill, item.agent, item.error);
        }
    }
    if report.removed.is_empty() && report.failed.is_empty() {
        println!("nothing to unlink");
    }
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

    fn tempdir(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "beck-unlink-tests-{name}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        base
    }

    fn write_manifest(beck: &std::path::Path, entries: Vec<Entry>) -> PathBuf {
        let mut mf = Manifest::empty();
        for e in entries {
            mf.add(e);
        }
        let path = beck.join(".beck-manifest.json");
        mf.save(&path).unwrap();
        path
    }

    fn sample_entry(skill: &str, agent: &str) -> Entry {
        Entry {
            skill: skill.into(),
            agent: agent.into(),
            target: PathBuf::from(format!("/tmp/fake/{skill}/{agent}/SKILL.md")),
            mode: InstallMode::Symlink,
            sha256: "deadbeef".into(),
            installed_at: "2026-04-11T00:00:00Z".into(),
        }
    }

    struct TrackingAdapter {
        name: &'static str,
        calls: RefCell<Vec<(String, String)>>,
        fail: bool,
    }

    unsafe impl Send for TrackingAdapter {}
    unsafe impl Sync for TrackingAdapter {}

    impl Adapter for TrackingAdapter {
        fn name(&self) -> &'static str {
            self.name
        }
        fn detect(&self) -> bool {
            true
        }
        fn target_root(&self) -> Result<PathBuf> {
            Ok(PathBuf::from("/tmp/mock"))
        }
        fn plan(
            &self,
            _skill: &beck::agents::skill::Skill,
        ) -> Result<beck::agents::adapter::InstallPlan> {
            unreachable!("plan not used in unlink tests")
        }
        fn install(&self, _plan: &beck::agents::adapter::InstallPlan) -> Result<Entry> {
            unreachable!("install not used in unlink tests")
        }
        fn uninstall(&self, entry: &Entry) -> Result<()> {
            self.calls
                .borrow_mut()
                .push((entry.skill.clone(), entry.agent.clone()));
            if self.fail {
                return Err(CliError::Validation(format!(
                    "mock uninstall failure for {}",
                    entry.skill
                )));
            }
            Ok(())
        }
    }

    #[test]
    fn refuses_without_scope() {
        let beck = tempdir("unlink-no-scope");
        let manifest_file = beck.join(".beck-manifest.json");
        let err = run_unlink(
            UnlinkArgs {
                skill: None,
                agent: None,
                all: false,
            },
            &manifest_file,
            vec![],
        )
        .expect_err("refuses");
        assert!(matches!(err, CliError::Validation(_)));
    }

    #[test]
    fn all_removes_every_entry() {
        let beck = tempdir("unlink-all");
        let manifest_file = write_manifest(
            &beck,
            vec![
                sample_entry("alpha", "claude-code"),
                sample_entry("beta", "claude-code"),
            ],
        );

        let adapter = TrackingAdapter {
            name: "claude-code",
            calls: RefCell::new(vec![]),
            fail: false,
        };

        let report = run_unlink(
            UnlinkArgs {
                skill: None,
                agent: None,
                all: true,
            },
            &manifest_file,
            vec![&adapter],
        )
        .unwrap();

        assert_eq!(report.removed.len(), 2);
        assert_eq!(adapter.calls.borrow().len(), 2);

        let mf = Manifest::load(&manifest_file).unwrap();
        assert!(mf.entries.is_empty());
    }

    #[test]
    fn skill_filter_only_touches_that_skill() {
        let beck = tempdir("unlink-by-skill");
        let manifest_file = write_manifest(
            &beck,
            vec![
                sample_entry("alpha", "claude-code"),
                sample_entry("beta", "claude-code"),
            ],
        );

        let adapter = TrackingAdapter {
            name: "claude-code",
            calls: RefCell::new(vec![]),
            fail: false,
        };

        let report = run_unlink(
            UnlinkArgs {
                skill: Some("beta".into()),
                agent: None,
                all: false,
            },
            &manifest_file,
            vec![&adapter],
        )
        .unwrap();

        assert_eq!(report.removed.len(), 1);
        assert_eq!(report.removed[0].skill, "beta");
        assert_eq!(adapter.calls.borrow().len(), 1);
        assert_eq!(adapter.calls.borrow()[0].0, "beta");

        let mf = Manifest::load(&manifest_file).unwrap();
        assert_eq!(mf.entries.len(), 1);
        assert_eq!(mf.entries[0].skill, "alpha");
    }

    #[test]
    fn failing_uninstall_surfaces_in_report() {
        let beck = tempdir("unlink-fail");
        let manifest_file = write_manifest(&beck, vec![sample_entry("alpha", "claude-code")]);

        let adapter = TrackingAdapter {
            name: "claude-code",
            calls: RefCell::new(vec![]),
            fail: true,
        };

        let report = run_unlink(
            UnlinkArgs {
                skill: None,
                agent: None,
                all: true,
            },
            &manifest_file,
            vec![&adapter],
        )
        .unwrap();

        assert!(report.removed.is_empty());
        assert_eq!(report.failed.len(), 1);

        let mf = Manifest::load(&manifest_file).unwrap();
        assert_eq!(mf.entries.len(), 1);
    }

    #[test]
    fn missing_manifest_is_not_an_error() {
        let beck = tempdir("unlink-no-manifest");
        let manifest_file = beck.join(".beck-manifest.json");
        let report = run_unlink(
            UnlinkArgs {
                skill: None,
                agent: None,
                all: true,
            },
            &manifest_file,
            vec![],
        )
        .unwrap();
        assert!(report.removed.is_empty());
        assert!(report.failed.is_empty());
    }

    #[test]
    fn unregistered_agent_prunes_without_calling_uninstall() {
        let beck = tempdir("unlink-stale-agent");
        let manifest_file = write_manifest(&beck, vec![sample_entry("alpha", "cursor-v0")]);

        let adapter = TrackingAdapter {
            name: "claude-code",
            calls: RefCell::new(vec![]),
            fail: false,
        };

        let report = run_unlink(
            UnlinkArgs {
                skill: None,
                agent: None,
                all: true,
            },
            &manifest_file,
            vec![&adapter],
        )
        .unwrap();

        assert_eq!(report.removed.len(), 1);
        assert!(adapter.calls.borrow().is_empty());

        let mf = Manifest::load(&manifest_file).unwrap();
        assert!(mf.entries.is_empty());
    }

    #[test]
    fn json_report_serializes_unlink_report() {
        let report = UnlinkReport {
            removed: vec![RemovedItem {
                skill: "caveman".into(),
                agent: "claude-code".into(),
                target: "/tmp/x/SKILL.md".into(),
            }],
            failed: vec![],
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"removed\""));
        assert!(json.contains("caveman"));
    }
}
