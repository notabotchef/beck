use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Frontmatter {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Parse SKILL.md contents into (frontmatter, body).
/// Accepts files with or without frontmatter. Unknown fields are ignored.
pub fn parse(contents: &str) -> (Frontmatter, String) {
    let trimmed = contents.trim_start_matches('\u{feff}');
    if !trimmed.starts_with("---") {
        return (Frontmatter::default(), contents.to_string());
    }
    let after = &trimmed[3..];
    let after = after.trim_start_matches(['\r', '\n']);
    if let Some(end) = find_closing_fence(after) {
        let yaml = &after[..end];
        let body_start = &after[end..];
        let body = body_start
            .trim_start_matches("---")
            .trim_start_matches(['\r', '\n'])
            .to_string();
        let fm: Frontmatter = serde_yaml::from_str(yaml).unwrap_or_default();
        (fm, body)
    } else {
        (Frontmatter::default(), contents.to_string())
    }
}

fn find_closing_fence(s: &str) -> Option<usize> {
    // Look for a line that is exactly "---" (possibly with \r).
    let mut idx = 0usize;
    for line in s.split_inclusive('\n') {
        let stripped = line.trim_end_matches(['\r', '\n']);
        if stripped == "---" {
            return Some(idx);
        }
        idx += line.len();
    }
    None
}
