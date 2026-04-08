//! Phase 0 eval harness.
//!
//! Reads tests/eval/queries.toml, builds an in-memory index over
//! tests/fixtures/skills/, runs every query, and reports top-1 and
//! top-3 recall. Exit code 0 always so the caller can read the number.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use beck::{db::Db, query::search, sync::sync_root};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct QuerySet {
    queries: Vec<Query>,
}

#[derive(Debug, Deserialize)]
struct Query {
    text: String,
    expected_top1: String,
    category: String,
}

fn repo_root() -> PathBuf {
    // eval is invoked from the workspace root typically; be defensive.
    let candidate = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    // If we were launched from crates/beck-core, walk up.
    let mut cur = candidate;
    for _ in 0..4 {
        if cur.join("tests/fixtures/skills").exists() && cur.join("tests/eval").exists() {
            return cur;
        }
        if !cur.pop() {
            break;
        }
    }
    PathBuf::from(".")
}

fn main() -> Result<()> {
    let root = repo_root();
    let fixtures = root.join("tests/fixtures/skills");
    let queries_path = root.join("tests/eval/queries.toml");

    let db = Db::in_memory()?;
    let indexed = sync_root(&db, &fixtures)?;
    println!("indexed {} skills from {}", indexed, fixtures.display());

    let raw = std::fs::read_to_string(&queries_path)
        .with_context(|| format!("read {}", queries_path.display()))?;
    let qs: QuerySet = toml::from_str(&raw)?;
    println!("running {} queries", qs.queries.len());

    let mut top1_hits = 0usize;
    let mut top3_hits = 0usize;
    let mut misses: Vec<(String, String, Vec<String>)> = Vec::new();
    let mut by_cat: std::collections::BTreeMap<String, (usize, usize, usize)> =
        Default::default();

    for q in &qs.queries {
        let results = search(&db, &q.text, 3)?;
        let names: Vec<String> = results.iter().map(|m| m.name.clone()).collect();
        let t1 = names.first().map(|n| n == &q.expected_top1).unwrap_or(false);
        let t3 = names.iter().any(|n| n == &q.expected_top1);
        if t1 {
            top1_hits += 1;
        }
        if t3 {
            top3_hits += 1;
        } else {
            misses.push((q.text.clone(), q.expected_top1.clone(), names.clone()));
        }
        let entry = by_cat.entry(q.category.clone()).or_insert((0, 0, 0));
        entry.0 += 1;
        if t1 {
            entry.1 += 1;
        }
        if t3 {
            entry.2 += 1;
        }
    }

    let total = qs.queries.len() as f64;
    let top1 = top1_hits as f64 / total * 100.0;
    let top3 = top3_hits as f64 / total * 100.0;

    println!();
    println!("=== Phase 0 eval results ===");
    println!("top-1 recall: {}/{} = {:.1}%", top1_hits, qs.queries.len(), top1);
    println!("top-3 recall: {}/{} = {:.1}%", top3_hits, qs.queries.len(), top3);
    println!();
    println!("by category:");
    for (cat, (n, t1, t3)) in &by_cat {
        println!(
            "  {:12} n={:2}  top-1={:.1}%  top-3={:.1}%",
            cat,
            n,
            (*t1 as f64 / *n as f64) * 100.0,
            (*t3 as f64 / *n as f64) * 100.0,
        );
    }
    println!();
    println!("gate: top-3 >= 85% required to proceed FTS5-only");
    if top3 >= 85.0 {
        println!("VERDICT: PASS. Proceed to Phase 1 as planned.");
    } else if top3 >= 65.0 {
        println!("VERDICT: PASS WITH NARRATIVE PIVOT. Reframe launch as '10x token reduction'.");
    } else {
        println!("VERDICT: FAIL. Reopen scope (embeddings in v0).");
    }

    if !misses.is_empty() {
        println!();
        println!("misses ({}):", misses.len());
        for (text, expected, got) in &misses {
            println!("  '{}' -> want {}  got {:?}", text, expected, got);
        }
    }

    let _ = Path::new(&fixtures); // silence unused import in some configs
    Ok(())
}
