//! beck-core: the shared indexing, query, and model logic.
//! Re-exported so both the `beck` binary and the `eval` harness can use them.

pub mod consts;
pub mod db;
pub mod error;
pub mod frontmatter;
pub mod paths;
pub mod query;
pub mod sync;

pub use db::Db;
pub use error::{CliError, Result};
pub use frontmatter::Frontmatter;
pub use query::{Match, search};
