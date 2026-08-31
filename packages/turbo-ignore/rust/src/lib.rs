//! Rust implementation of the `turbo-ignore` decision engine.
//!
//! The public API keeps the TypeScript implementation's fail-open contract:
//! an analysis failure returns [`BuildDecision::Deploy`], never `Skip`.

mod commit;
mod comparison;
mod discovery;
mod engine;
mod errors;
mod json5;
mod model;
mod process;
mod reporter;
mod sanitize;
mod tools;
mod validation;

pub use commit::{FORCE_ALL_COMMITS, ONLY_WORKSPACE_PREFIX, SKIP_ALL_COMMITS, check_commit};
pub use comparison::{Comparison, ComparisonKind, get_comparison};
pub use discovery::{find_turbo_root, get_workspace, infer_turbo_version};
pub use engine::{Environment, Options, evaluate};
pub use errors::{ErrorClassification, ErrorCode, ErrorLevel, classify_error};
pub use json5::{Json5ScanError, top_level_keys};
pub use model::{BuildDecision, CommitDecision, CommitResult, CommitScope, EXIT_DEPLOY, EXIT_SKIP};
pub use process::{CommandOutput, CommandRunner, CommandSpec, ProcessError, SystemCommandRunner};
pub use reporter::{ConsoleReporter, Reporter};
pub use sanitize::sanitize_for_log;
pub use tools::{ToolError, resolve_git, resolve_turbo};
pub use validation::{
    InputError, validate_ref, validate_task, validate_text_field, validate_version_selector,
    validate_workspace,
};
