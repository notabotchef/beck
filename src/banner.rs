//! Minimal ASCII banner for beck.
//! Two tones: default (white) and dim (gray). Only shown on TTY.

use std::io::IsTerminal;

const BANNER: &str = r#"
 ┌─● ● ●──────────────────┐
 │ █   █ ████  ███  █   █ │
 │ ██ ██ █    █   █ █ █ █ │
 │ █ █ █ ███  █   █ █ █ █ │
 │ █   █ █    █   █ █ █ █ │
 │ █   █ ████  ███   █ █  │
 └─────────────────────────┘
  your agent's skills, at its beck and call.
"#;

/// Print the banner to stderr if stdout is a TTY (not piped).
pub fn maybe_print() {
    if std::io::stdout().is_terminal() {
        let lines: Vec<&str> = BANNER.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                eprintln!();
            } else if i < lines.len() - 2 {
                // ASCII art lines: dim gray
                eprintln!("\x1b[2m{}\x1b[0m", line);
            } else {
                // Tagline: default color
                eprintln!("{}", line);
            }
        }
    }
}
