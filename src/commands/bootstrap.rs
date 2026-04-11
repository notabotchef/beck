//! `beck bootstrap` — initialize the user's beck home.
//!
//! Creates `<beck_home>/skills/` and writes an empty manifest if one does
//! not already exist. Idempotent: a second run is a no-op and exits 0.

use serde_json::json;
use std::fs;

use beck::agents::manifest::Manifest;
use beck::agents::paths::{beck_home, manifest_path, skills_home};
use beck::error::Result;

pub async fn handle(json_out: bool) -> Result<()> {
    let home = beck_home()?;
    let skills = skills_home()?;
    let manifest = manifest_path()?;

    fs::create_dir_all(&skills)?;

    let manifest_already_existed = manifest.exists();
    if !manifest_already_existed {
        Manifest::empty().save(&manifest)?;
    }

    if json_out {
        let payload = json!({
            "beck_home": home,
            "skills_home": skills,
            "manifest_path": manifest,
            "created": !manifest_already_existed,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    } else if manifest_already_existed {
        println!("already initialized at {}", home.display());
    } else {
        println!("initialized beck home at {}", home.display());
        println!("  skills: {}", skills.display());
        println!("  manifest: {}", manifest.display());
    }

    Ok(())
}
