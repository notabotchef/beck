//! Typed error with distinct exit codes.
//! Adapted from mateonunez/nucleo's src/error.rs.

use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Validation(String),

    #[error(transparent)]
    Db(#[from] rusqlite::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl CliError {
    pub const EXIT_CODE_NOT_FOUND: i32 = 1;
    pub const EXIT_CODE_VALIDATION: i32 = 3;
    pub const EXIT_CODE_DB: i32 = 4;
    pub const EXIT_CODE_IO: i32 = 5;
    pub const EXIT_CODE_OTHER: i32 = 6;

    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::NotFound(_) => Self::EXIT_CODE_NOT_FOUND,
            CliError::Validation(_) => Self::EXIT_CODE_VALIDATION,
            CliError::Db(_) => Self::EXIT_CODE_DB,
            CliError::Io(_) => Self::EXIT_CODE_IO,
            CliError::Other(_) => Self::EXIT_CODE_OTHER,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        let (code, reason) = match self {
            CliError::NotFound(_) => (404, "notFound"),
            CliError::Validation(_) => (400, "validationError"),
            CliError::Db(_) => (500, "dbError"),
            CliError::Io(_) => (500, "ioError"),
            CliError::Other(_) => (500, "internalError"),
        };
        json!({
            "error": {
                "code": code,
                "message": format!("{self}"),
                "reason": reason,
            }
        })
    }
}

pub fn print_error_json(err: &CliError) {
    let json = err.to_json();
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&json).unwrap_or_default()
    );
}

pub type Result<T> = std::result::Result<T, CliError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_codes_are_distinct() {
        let codes = [
            CliError::EXIT_CODE_NOT_FOUND,
            CliError::EXIT_CODE_VALIDATION,
            CliError::EXIT_CODE_DB,
            CliError::EXIT_CODE_IO,
            CliError::EXIT_CODE_OTHER,
        ];
        let unique: std::collections::HashSet<i32> = codes.iter().copied().collect();
        assert_eq!(unique.len(), codes.len());
    }

    #[test]
    fn not_found_exit_code() {
        let e = CliError::NotFound("whisper".into());
        assert_eq!(e.exit_code(), CliError::EXIT_CODE_NOT_FOUND);
        assert_eq!(e.to_json()["error"]["code"], 404);
    }

    #[test]
    fn validation_exit_code() {
        let e = CliError::Validation("empty query".into());
        assert_eq!(e.exit_code(), CliError::EXIT_CODE_VALIDATION);
    }
}
