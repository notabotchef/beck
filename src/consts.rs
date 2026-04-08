//! Identity constants. The only place to change these when forking.
//! Mirrors the 4-constant pattern from mateonunez/nucleo.

/// Application name, used in --help, MCP server info, and User-Agent strings.
pub const APP_NAME: &str = "beck";

/// Config and data directory name under XDG paths.
pub const APP_DIR: &str = "beck";

/// Environment variable prefix. All env vars: `<PREFIX>_FOO`.
#[allow(dead_code)]
pub const APP_PREFIX: &str = "BECK";

/// Binary name for subprocess resolution and error messages.
#[allow(dead_code)]
pub const APP_BIN: &str = "beck";
