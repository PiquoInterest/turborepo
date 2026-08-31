use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::Deserialize;
use semver::Version;

use crate::{
    BuildDecision, CommandRunner, CommandSpec, CommitResult, ErrorLevel, Reporter,
    classify_error, check_commit, find_turbo_root, get_comparison, get_workspace,
    infer_turbo_version, resolve_git, resolve_turbo, sanitize_for_log, validate_ref,
    validate_task, validate_version_selector, validate_workspace,
};

const DEFAULT_MAX_OUTPUT_BYTES: usize = 1_024 * 1_024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_OUTPUT_BYTES: usize = 64 * 1_024 * 1_024;
const MAX_TIMEOUT: Duration = Duration::from_secs(30 * 60);
const MAX_COMMIT_MESSAGE_BYTES: usize = 1_024 * 1_024;
const MAX_LOGGED_DEPENDENCIES: usize = 20;

#[derive(Debug, Clone)]
pub struct Options {
    pub workspace: Option<String>,
    pub task: Option<String>,
    pub fallback: Option<String>,
    pub directory: Option<PathBuf>,
    pub turbo_version: Option<String>,
    pub turbo_path: Option<PathBuf>,
    pub git_path: Option<PathBuf>,
    pub max_output_bytes: usize,
    pub timeout: Duration,
    /// Test seam. Production callers leave this as `None`.
    pub current_directory: Option<PathBuf>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            workspace: None,
            task: None,
            fallback: None,
            directory: None,
            turbo_version: None,
            turbo_path: None,
            git_path: None,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
            timeout: DEFAULT_TIMEOUT,
            current_directory: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Environment {
    pub vercel: bool,
    pub turbo_force: bool,
    pub commit_message: Option<String>,
    pub previous_sha: Option<String>,
    pub git_commit_ref: Option<String>,
}

impl Environment {
    #[must_use]
    pub fn from_process() -> Self {
        fn non_empty(name: &str) -> Option<String> {
            env::var(name).ok().filter(|value| !value.is_empty())
        }

        Self {
            vercel: env::var("VERCEL").is_ok_and(|value| value == "1"),
            turbo_force: env::var("TURBO_FORCE").is_ok_and(|value| value == "true"),
            commit_message: non_empty("VERCEL_GIT_COMMIT_MESSAGE"),
            previous_sha: non_empty("VERCEL_GIT_PREVIOUS_SHA"),
            git_commit_ref: non_empty("VERCEL_GIT_COMMIT_REF"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct DryRun {
    packages: Option<Vec<String>>,
}

fn final_decision(decision: BuildDecision, reporter: &dyn Reporter) -> BuildDecision {
    match decision {
        BuildDecision::Skip => reporter.log("⏭ Ignoring the change"),
        BuildDecision::Deploy => reporter.log("✓ Proceeding with deployment"),
    }
    decision
}

fn current_directory(options: &Options, reporter: &dyn Reporter) -> Option<PathBuf> {
    let base = match options.current_directory.clone() {
        Some(path) => path,
        None => match env::current_dir() {
            Ok(path) => path,
            Err(error) => {
                reporter.error(&format!("Could not read current directory: {error}"));
                return None;
            }
        },
    };

    let Some(configured) = options.directory.as_ref() else {
        return Some(base);
    };
    let resolved = if configured.is_absolute() {
        configured.clone()
    } else {
        base.join(configured)
    };

    if resolved.exists() {
        return match fs::canonicalize(&resolved) {
            Ok(path) if path.is_dir() => Some(path),
            Ok(path) => {
                reporter.error(&format!(
                    "Directory \"{}\" is not a directory",
                    sanitize_for_log(&path.display().to_string())
                ));
                None
            }
            Err(error) => {
                reporter.error(&format!(
                    "Directory \"{}\" could not be canonicalized: {error}",
                    sanitize_for_log(&resolved.display().to_string())
                ));
                None
            }
        };
    }

    reporter.warn(&format!(
        "Directory \"{}\" does not exist, using current directory",
        sanitize_for_log(&configured.display().to_string())
    ));
    Some(base)
}
