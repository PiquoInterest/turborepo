use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use semver::Version;
use serde::Deserialize;

use crate::{
    BuildDecision, CommandRunner, CommandSpec, CommitResult, ErrorLevel, Reporter, check_commit,
    classify_error, find_turbo_root, get_comparison, get_workspace, infer_turbo_version,
    resolve_git, resolve_turbo, sanitize_for_log, validate_ref, validate_task,
    validate_version_selector, validate_workspace,
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

fn get_commit_message(
    environment: &Environment,
    git: Option<&Path>,
    root: &Path,
    runner: &dyn CommandRunner,
    options: &Options,
    reporter: &dyn Reporter,
) -> Option<String> {
    if environment.vercel
        && let Some(message) = environment.commit_message.as_ref()
    {
        if message.len() > MAX_COMMIT_MESSAGE_BYTES {
            reporter.error("Vercel commit message exceeds the 1 MiB safety limit");
            return None;
        }
        return Some(message.clone());
    }

    let Some(git) = git else {
        reporter.error("Trusted Git executable is required to read the commit message");
        return None;
    };

    let spec = CommandSpec {
        program: git.to_path_buf(),
        args: vec![
            OsString::from("show"),
            OsString::from("-s"),
            OsString::from("--format=%B"),
        ],
        cwd: root.to_path_buf(),
        timeout: options.timeout,
        max_output_bytes: options.max_output_bytes,
    };

    match runner.run(&spec) {
        Ok(output) if output.status.success() => {
            if output.stdout.len() > MAX_COMMIT_MESSAGE_BYTES {
                reporter.error("Git commit message exceeds the 1 MiB safety limit");
                None
            } else {
                Some(output.stdout)
            }
        }
        Ok(output) => {
            let detail = if output.stderr.trim().is_empty() {
                output.stdout
            } else {
                output.stderr
            };
            reporter.error(&format!(
                "Could not read the Git commit message: {}",
                sanitize_for_log(detail.trim())
            ));
            None
        }
        Err(error) => {
            reporter.error(&format!("Could not read the Git commit message: {error}"));
            None
        }
    }
}

fn selected_task<'a>(options: &'a Options, reporter: &dyn Reporter) -> &'a str {
    match options.task.as_deref() {
        Some(task) => {
            reporter.info(&format!(
                "Using \"{}\" as the task from the arguments",
                sanitize_for_log(task)
            ));
            task
        }
        None => {
            reporter.info("Using \"build\" as the task as it was unspecified");
            "build"
        }
    }
}

fn validate_base_inputs(workspace: &str, task: &str, reporter: &dyn Reporter) -> bool {
    if let Err(error) = validate_workspace(workspace) {
        reporter.error(&error.to_string());
        return false;
    }
    if let Err(error) = validate_task(task) {
        reporter.error(&error.to_string());
        return false;
    }

    true
}

fn validate_analysis_inputs(
    fallback: Option<&str>,
    turbo_version: Option<&str>,
    max_output_bytes: usize,
    timeout: Duration,
    reporter: &dyn Reporter,
) -> bool {
    if !(1_024..=MAX_OUTPUT_BYTES).contains(&max_output_bytes) {
        reporter.error(&format!(
            "max output must be between 1024 and {MAX_OUTPUT_BYTES} bytes"
        ));
        return false;
    }
    if timeout.is_zero() || timeout > MAX_TIMEOUT {
        reporter.error("timeout must be between 1 second and 30 minutes");
        return false;
    }
    if let Some(reference) = fallback
        && let Err(error) = validate_ref(reference)
    {
        reporter.error(&format!("Invalid fallback ref: {error}"));
        return false;
    }

    if let Some(version) = turbo_version
        && let Err(error) = validate_version_selector(version)
    {
        reporter.error(&format!(
            "Refusing unsafe or unsupported turbo version selector \"{}\": {error}",
            sanitize_for_log(version)
        ));
        return false;
    }

    true
}

fn display_command(turbo: &Path, task: &str, filter: &str) -> String {
    format!(
        "{} run {} --filter=\"{}\" --dry=json",
        sanitize_for_log(&turbo.display().to_string()),
        sanitize_for_log(task),
        sanitize_for_log(filter)
    )
}

fn reported_turbo_version(output: &str) -> Option<Version> {
    output.split_whitespace().find_map(|candidate| {
        let candidate =
            candidate.trim_matches(|character: char| matches!(character, '"' | '\'' | ',' | ';'));
        let candidate = candidate.strip_prefix('v').unwrap_or(candidate);
        Version::parse(candidate).ok()
    })
}

fn verify_turbo_version(
    turbo: &Path,
    selector: Option<&str>,
    root: &Path,
    runner: &dyn CommandRunner,
    options: &Options,
    reporter: &dyn Reporter,
) -> bool {
    let Some(selector) = selector else {
        return true;
    };
    let requirement = match validate_version_selector(selector) {
        Ok(requirement) => requirement,
        Err(error) => {
            reporter.error(&format!(
                "Refusing unsafe or unsupported turbo version selector \"{}\": {error}",
                sanitize_for_log(selector)
            ));
            return false;
        }
    };
    let spec = CommandSpec {
        program: turbo.to_path_buf(),
        args: vec![OsString::from("--version")],
        cwd: root.to_path_buf(),
        timeout: options.timeout,
        max_output_bytes: options.max_output_bytes.min(64 * 1_024),
    };

    let output = match runner.run(&spec) {
        Ok(output) => output,
        Err(error) => {
            reporter.error(&format!(
                "Could not verify the trusted Turbo binary: {error}"
            ));
            return false;
        }
    };
    if !output.status.success() {
        let detail = if output.stderr.trim().is_empty() {
            output.stdout.trim()
        } else {
            output.stderr.trim()
        };
        reporter.error(&format!(
            "Could not verify the trusted Turbo binary: {}",
            sanitize_for_log(detail)
        ));
        return false;
    }

    let version =
        reported_turbo_version(&output.stdout).or_else(|| reported_turbo_version(&output.stderr));
    let Some(version) = version else {
        reporter.error("Trusted Turbo binary returned an unrecognized version string");
        return false;
    };
    if !requirement.matches(&version) {
        reporter.error(&format!(
            "Trusted Turbo binary version {version} does not satisfy requested selector {}",
            sanitize_for_log(selector)
        ));
        return false;
    }

    reporter.info(&format!(
        "Using trusted Turbo version {version} from \"{}\"",
        sanitize_for_log(&turbo.display().to_string())
    ));
    true
}

fn parse_dry_run(
    stdout: &str,
    command: &str,
    workspace: &str,
    reporter: &dyn Reporter,
) -> BuildDecision {
    let parsed = match serde_json::from_str::<Option<DryRun>>(stdout) {
        Ok(Some(value)) => value,
        Ok(None) | Err(_) => {
            reporter.error(&format!("Failed to parse JSON output from `{command}`."));
            return BuildDecision::Deploy;
        }
    };

    let Some(packages) = parsed.packages else {
        reporter.info("Detected single package repo");
        return BuildDecision::Deploy;
    };

    if packages.is_empty() {
        reporter.info("This project and its dependencies are not affected");
        return BuildDecision::Skip;
    }

    if packages.len() == 1 {
        reporter.info(&format!(
            "This commit affects \"{}\"",
            sanitize_for_log(workspace)
        ));
    } else {
        let dependencies: Vec<String> = packages
            .iter()
            .skip(1)
            .take(MAX_LOGGED_DEPENDENCIES)
            .map(|package| sanitize_for_log(package))
            .collect();
        let count = packages.len() - 1;
        let noun = if count == 1 {
            "dependency"
        } else {
            "dependencies"
        };
        let omitted = count.saturating_sub(dependencies.len());
        let displayed = if omitted == 0 {
            dependencies.join(", ")
        } else {
            format!("{}, … +{omitted} more", dependencies.join(", "))
        };
        reporter.info(&format!(
            "This commit affects \"{}\" and {count} {noun} ({})",
            sanitize_for_log(workspace),
            displayed
        ));
    }

    BuildDecision::Deploy
}

/// Evaluates whether deployment may be skipped.
///
/// Every infrastructure, parsing, validation, or subprocess error returns
/// [`BuildDecision::Deploy`]. This is the security-critical fail-open
/// invariant.
#[must_use]
pub fn evaluate(
    options: &Options,
    environment: &Environment,
    runner: &dyn CommandRunner,
    reporter: &dyn Reporter,
) -> BuildDecision {
    if environment.vercel {
        reporter.warn(
            "\"turbo-ignore\" is deprecated. Use Vercel's built-in project skipping instead.",
        );
        reporter.warn("Learn more: https://vercel.com/docs/monorepos#skipping-unaffected-projects");
    } else {
        reporter.warn("\"turbo-ignore\" is deprecated. Use \"turbo query affected\" instead.");
        reporter.warn(
            "Learn more: https://turborepo.dev/docs/reference/query#migrating-from-turbo-ignore",
        );
    }
    reporter.info("Using Turborepo to determine if this project is affected by the commit...");

    let Some(directory) = current_directory(options, reporter) else {
        return final_decision(BuildDecision::Deploy, reporter);
    };

    if environment.turbo_force {
        reporter.info("`TURBO_FORCE` detected");
        return final_decision(BuildDecision::Deploy, reporter);
    }
    let Some(root) = find_turbo_root(&directory) else {
        reporter.error("Monorepo root not found. turbo-ignore inferencing failed");
        return final_decision(BuildDecision::Deploy, reporter);
    };
    let Some(workspace) = get_workspace(options.workspace.as_deref(), &directory, reporter) else {
        return final_decision(BuildDecision::Deploy, reporter);
    };
    let turbo_version = infer_turbo_version(options.turbo_version.as_deref(), &root, reporter);
    let task = selected_task(options, reporter);

    // Vercel provides the authoritative commit message directly. Resolving Git
    // before consulting it would make proven skip/deploy directives depend on a
    // tool that the TypeScript implementation does not invoke on this path.
    let mut git = if environment.vercel && environment.commit_message.is_some() {
        None
    } else {
        match resolve_git(options.git_path.as_deref()) {
            Ok(path) => Some(path),
            Err(error) => {
                reporter.error(&error.to_string());
                return final_decision(BuildDecision::Deploy, reporter);
            }
        }
    };

    let Some(commit_message) = get_commit_message(
        environment,
        git.as_deref(),
        &root,
        runner,
        options,
        reporter,
    ) else {
        return final_decision(BuildDecision::Deploy, reporter);
    };
    let commit = check_commit(&workspace, &commit_message);
    match commit.result {
        CommitResult::Skip => {
            reporter.info(&commit.reason);
            return final_decision(BuildDecision::Skip, reporter);
        }
        CommitResult::Deploy => {
            reporter.info(&commit.reason);
            return final_decision(BuildDecision::Deploy, reporter);
        }
        CommitResult::Conflict => {
            reporter.info(&commit.reason);
            return final_decision(BuildDecision::Deploy, reporter);
        }
        CommitResult::Continue => {}
    }

    // The TypeScript implementation evaluates commit directives before using
    // workspace/task values in a Turbo filter. Keep those early decisions, then
    // validate every value before any analysis subprocess is started.
    if !validate_base_inputs(&workspace, task, reporter) {
        return final_decision(BuildDecision::Deploy, reporter);
    }

    if !validate_analysis_inputs(
        options.fallback.as_deref(),
        turbo_version.as_deref(),
        options.max_output_bytes,
        options.timeout,
        reporter,
    ) {
        return final_decision(BuildDecision::Deploy, reporter);
    }

    // Git is only needed on Vercel when a previous deployment object must be
    // validated. A missing Git executable makes that object unreachable, which
    // preserves the TypeScript fallback behavior instead of suppressing a build.
    if environment.vercel && environment.previous_sha.is_some() && git.is_none() {
        match resolve_git(options.git_path.as_deref()) {
            Ok(path) => git = Some(path),
            Err(error) => reporter.warn(&error.to_string()),
        }
    }

    let comparison = get_comparison(
        &workspace,
        options.fallback.as_deref(),
        environment,
        git.as_deref(),
        &root,
        runner,
        reporter,
        options.timeout,
        options.max_output_bytes,
    );
    let Some(comparison) = comparison else {
        return final_decision(BuildDecision::Deploy, reporter);
    };

    let turbo = match resolve_turbo(&root, options.turbo_path.as_deref()) {
        Ok(path) => path,
        Err(error) => {
            reporter.error(&error.to_string());
            return final_decision(BuildDecision::Deploy, reporter);
        }
    };
    if !verify_turbo_version(
        &turbo,
        turbo_version.as_deref(),
        &root,
        runner,
        options,
        reporter,
    ) {
        return final_decision(BuildDecision::Deploy, reporter);
    }

    let filter = format!("{workspace}...[{}]", comparison.reference);
    let command = display_command(&turbo, task, &filter);
    reporter.info(&format!("Analyzing results of `{command}`"));

    let spec = CommandSpec {
        program: turbo,
        args: vec![
            OsString::from("run"),
            OsString::from(task),
            OsString::from(format!("--filter={filter}")),
            OsString::from("--dry=json"),
        ],
        cwd: root,
        timeout: options.timeout,
        max_output_bytes: options.max_output_bytes,
    };

    let output = match runner.run(&spec) {
        Ok(output) => output,
        Err(error) => {
            reporter.error(&error.to_string());
            return final_decision(BuildDecision::Deploy, reporter);
        }
    };

    if !output.status.success() {
        let detail = if output.stderr.trim().is_empty() {
            output.stdout.trim()
        } else {
            output.stderr.trim()
        };
        let classification = classify_error(detail);
        match classification.level {
            ErrorLevel::Warn => reporter.warn(&classification.message),
            ErrorLevel::Error => reporter.error(&format!(
                "{}: {}",
                classification.code.as_str(),
                sanitize_for_log(&classification.message)
            )),
        }
        return final_decision(BuildDecision::Deploy, reporter);
    }

    final_decision(
        parse_dry_run(&output.stdout, &command, &workspace, reporter),
        reporter,
    )
}
