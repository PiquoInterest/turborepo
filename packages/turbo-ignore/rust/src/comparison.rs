use std::{ffi::OsString, path::Path, time::Duration};

use crate::{CommandRunner, CommandSpec, Environment, Reporter, sanitize_for_log, validate_ref};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonKind {
    PreviousDeploy,
    HeadRelative,
    CustomFallback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    pub reference: String,
    pub kind: ComparisonKind,
}

fn fallback(reference: Option<&str>, reporter: &dyn Reporter) -> Option<Comparison> {
    let reference = reference?;
    if let Err(error) = validate_ref(reference) {
        reporter.error(&format!("Invalid fallback ref: {error}"));
        return None;
    }
    reporter.info(&format!(
        "Falling back to ref {}",
        sanitize_for_log(reference)
    ));
    Some(Comparison {
        reference: reference.to_owned(),
        kind: ComparisonKind::CustomFallback,
    })
}

fn validate_object(
    reference: &str,
    git: Option<&Path>,
    root: &Path,
    runner: &dyn CommandRunner,
    timeout: Duration,
    max_output_bytes: usize,
) -> bool {
    if validate_ref(reference).is_err() {
        return false;
    }
    let Some(git) = git else {
        return false;
    };

    let spec = CommandSpec {
        program: git.to_path_buf(),
        args: vec![
            OsString::from("cat-file"),
            OsString::from("-e"),
            OsString::from("--end-of-options"),
            OsString::from(format!("{reference}^{{object}}")),
        ],
        cwd: root.to_path_buf(),
        timeout,
        max_output_bytes,
    };

    runner
        .run(&spec)
        .is_ok_and(|output| output.status.success())
}

#[allow(clippy::too_many_arguments)]
pub fn get_comparison(
    workspace: &str,
    fallback_ref: Option<&str>,
    environment: &Environment,
    git: Option<&Path>,
    root: &Path,
    runner: &dyn CommandRunner,
    reporter: &dyn Reporter,
    timeout: Duration,
    max_output_bytes: usize,
) -> Option<Comparison> {
    let workspace = sanitize_for_log(workspace);
    let branch_suffix = environment
        .git_commit_ref
        .as_deref()
        .map(sanitize_for_log)
        .map(|branch| format!(" on branch \"{branch}\""))
        .unwrap_or_default();

    if environment.vercel {
        if let Some(previous) = environment.previous_sha.as_deref() {
            if validate_object(previous, git, root, runner, timeout, max_output_bytes) {
                reporter.info(&format!(
                    "Found previous deployment (\"{}\") for \"{workspace}\"{branch_suffix}",
                    sanitize_for_log(previous)
                ));
                return Some(Comparison {
                    reference: previous.to_owned(),
                    kind: ComparisonKind::PreviousDeploy,
                });
            }

            reporter.info(&format!(
                "Previous deployment (\"{}\") for \"{workspace}\"{branch_suffix} is unreachable.",
                sanitize_for_log(previous)
            ));
            return fallback(fallback_ref, reporter);
        }

        reporter.info(&format!(
            "No previous deployments found for \"{workspace}\"{branch_suffix}"
        ));
        return fallback(fallback_ref, reporter);
    }

    if fallback_ref.is_some() {
        return fallback(fallback_ref, reporter);
    }

    Some(Comparison {
        reference: "HEAD^".to_owned(),
        kind: ComparisonKind::HeadRelative,
    })
}
