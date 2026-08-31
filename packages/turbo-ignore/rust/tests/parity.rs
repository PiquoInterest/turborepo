use std::{
    collections::VecDeque, error::Error, ffi::OsString, fs, path::Path, process::ExitStatus,
    sync::Mutex, time::Duration,
};

use pretty_assertions::assert_eq;
use tempfile::TempDir;
use turbo_ignore::{
    BuildDecision, CommandOutput, CommandRunner, CommandSpec, CommitResult, CommitScope,
    ComparisonKind, Environment, ErrorCode, ErrorLevel, FORCE_ALL_COMMITS, Options, ProcessError,
    Reporter, SKIP_ALL_COMMITS, check_commit, classify_error, evaluate, find_turbo_root,
    get_comparison, get_workspace, infer_turbo_version, top_level_keys,
};

#[cfg(unix)]
fn exit_status(success: bool) -> ExitStatus {
    use std::os::unix::process::ExitStatusExt as _;
    ExitStatus::from_raw(if success { 0 } else { 1 << 8 })
}

#[cfg(windows)]
fn exit_status(success: bool) -> ExitStatus {
    use std::os::windows::process::ExitStatusExt as _;
    ExitStatus::from_raw(if success { 0 } else { 1 })
}

#[derive(Debug)]
struct MockRunner {
    responses: Mutex<VecDeque<CommandOutput>>,
    calls: Mutex<Vec<CommandSpec>>,
}

impl MockRunner {
    fn new(responses: impl IntoIterator<Item = CommandOutput>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().collect()),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn call_count(&self) -> Result<usize, Box<dyn Error>> {
        let calls = self
            .calls
            .lock()
            .map_err(|_| std::io::Error::other("calls mutex poisoned"))?;
        Ok(calls.len())
    }

    fn calls(&self) -> Result<Vec<CommandSpec>, Box<dyn Error>> {
        let calls = self
            .calls
            .lock()
            .map_err(|_| std::io::Error::other("calls mutex poisoned"))?;
        Ok(calls.clone())
    }
}

impl CommandRunner for MockRunner {
    fn run(&self, spec: &CommandSpec) -> Result<CommandOutput, ProcessError> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push(spec.clone());
        }
        let mut responses =
            self.responses
                .lock()
                .map_err(|_| ProcessError::ReaderChannelClosed {
                    program: spec.program.clone(),
                })?;
        responses
            .pop_front()
            .ok_or_else(|| ProcessError::ReaderChannelClosed {
                program: spec.program.clone(),
            })
    }
}

#[derive(Debug, Default)]
struct RecordingReporter {
    messages: Mutex<Vec<(String, String)>>,
}

impl RecordingReporter {
    fn push(&self, level: &str, message: &str) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.push((level.to_owned(), message.to_owned()));
        }
    }

    fn contains(&self, needle: &str) -> Result<bool, Box<dyn Error>> {
        let messages = self
            .messages
            .lock()
            .map_err(|_| std::io::Error::other("messages mutex poisoned"))?;
        Ok(messages.iter().any(|(_, message)| message.contains(needle)))
    }
}

impl Reporter for RecordingReporter {
    fn info(&self, message: &str) {
        self.push("info", message);
    }

    fn warn(&self, message: &str) {
        self.push("warn", message);
    }

    fn error(&self, message: &str) {
        self.push("error", message);
    }

    fn log(&self, message: &str) {
        self.push("log", message);
    }
}

fn write(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn make_executable(path: &Path) -> Result<(), Box<dyn Error>> {
    write(path, "#!/bin/sh\nexit 0\n")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

fn successful_output(stdout: &str) -> CommandOutput {
    CommandOutput {
        status: exit_status(true),
        stdout: stdout.to_owned(),
        stderr: String::new(),
    }
}

fn failed_output(stderr: &str) -> CommandOutput {
    CommandOutput {
        status: exit_status(false),
        stdout: String::new(),
        stderr: stderr.to_owned(),
    }
}

fn fixture() -> Result<(TempDir, Options), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let root = directory.path();
    write(
        &root.join("package.json"),
        r#"{"name":"root","devDependencies":{"turbo":"2.10.13-canary.1"}}"#,
    )?;
    write(&root.join("turbo.json"), r#"{"tasks":{"build":{}}}"#)?;
    write(&root.join("apps/web/package.json"), r#"{"name":"web"}"#)?;
    let git = root.join("tools/git");
    let turbo = root.join("tools/turbo");
    make_executable(&git)?;
    make_executable(&turbo)?;

    let options = Options {
        workspace: Some("web".to_owned()),
        task: Some("build".to_owned()),
        fallback: Some("main".to_owned()),
        directory: Some(root.join("apps/web")),
        turbo_version: None,
        turbo_path: Some(turbo),
        git_path: Some(git),
        max_output_bytes: 1_024 * 1_024,
        timeout: Duration::from_secs(2),
        current_directory: Some(root.to_path_buf()),
    };
    Ok((directory, options))
}

#[test]
fn global_skip_directives_match_typescript_contract() {
    for directive in SKIP_ALL_COMMITS {
        let decision = check_commit("web", &format!("subject\n\n{directive}"));
        assert_eq!(decision.result, CommitResult::Skip);
        assert_eq!(decision.scope, CommitScope::Global);
        assert_eq!(
            decision.reason,
            format!("Found commit message: {directive}")
        );
    }
}

#[test]
fn global_force_directives_match_typescript_contract() {
    for directive in FORCE_ALL_COMMITS {
        let decision = check_commit("web", &format!("subject\n\n{directive}"));
        assert_eq!(decision.result, CommitResult::Deploy);
        assert_eq!(decision.scope, CommitScope::Global);
        assert_eq!(
            decision.reason,
            format!("Found commit message: {directive}")
        );
    }
}

#[test]
fn workspace_directives_take_precedence_over_global_directives() {
    let decision = check_commit("web", "[skip ci] [vercel deploy web]");
    assert_eq!(decision.result, CommitResult::Deploy);
    assert_eq!(decision.scope, CommitScope::Workspace);

    let decision = check_commit("web", "[vercel deploy] [vercel skip web]");
    assert_eq!(decision.result, CommitResult::Skip);
    assert_eq!(decision.scope, CommitScope::Workspace);
}

#[test]
fn only_directive_matches_typescript_for_single_directive() {
    let deploy = check_commit("web", "feat: change [vercel only web]");
    assert_eq!(deploy.result, CommitResult::Deploy);
    assert_eq!(deploy.scope, CommitScope::Workspace);

    let skip = check_commit("web", "feat: change [vercel only api]");
    assert_eq!(skip.result, CommitResult::Skip);
    assert_eq!(skip.scope, CommitScope::Workspace);
}

#[test]
fn conflicting_directives_match_typescript_contract() {
    let workspace = check_commit("web", "[vercel deploy web] and [vercel skip web]");
    assert_eq!(workspace.result, CommitResult::Conflict);
    assert_eq!(workspace.scope, CommitScope::Workspace);

    let global = check_commit("web", "[vercel deploy] and [skip ci]");
    assert_eq!(global.result, CommitResult::Conflict);
    assert_eq!(global.scope, CommitScope::Global);
}

#[test]
fn no_directive_continues_analysis() {
    let decision = check_commit("web", "ordinary commit");
    assert_eq!(decision.result, CommitResult::Continue);
    assert_eq!(decision.scope, CommitScope::Global);
    assert_eq!(
        decision.reason,
        "No deploy or skip string found in commit message."
    );
}

#[test]
fn json5_scanner_recognizes_turbo_top_level_keys() -> Result<(), Box<dyn Error>> {
    let keys = top_level_keys(
        r#"
        // comment
        {
          extends: ["//"],
          'tasks': {
            build: { outputs: ["dist/**",], },
          },
        }
        "#,
    )?;
    assert_eq!(keys, vec!["extends", "tasks"]);
    Ok(())
}

#[test]
fn json5_scanner_validates_the_complete_document() {
    for invalid in [
        "{ tasks: { build: true } trailing: false }",
        "{ tasks: [1, 2 }",
        "{ tasks: unknownIdentifier }",
        "{ tasks: { build: '\\uD800' } }",
        "{ tasks: {} } garbage",
    ] {
        assert!(
            top_level_keys(invalid).is_err(),
            "malformed JSON5 should be rejected: {invalid}"
        );
    }
}

#[test]
fn json5_scanner_has_a_finite_nesting_limit() {
    let nested = format!("{{tasks:{}}}", "[".repeat(140) + &"]".repeat(140));
    assert!(top_level_keys(&nested).is_err());
}

#[test]
fn json5_scanner_accepts_supported_number_forms_and_rejects_ambiguous_ones() {
    for valid in [
        "{value:.5}",
        "{value:1.}",
        "{value:-1.25e+2}",
        "{value:0xCAFE}",
        "{value:+Infinity}",
    ] {
        assert!(
            top_level_keys(valid).is_ok(),
            "valid JSON5 rejected: {valid}"
        );
    }

    for invalid in [
        "{value:.}",
        "{value:--1}",
        "{value:1e}",
        "{value:0x}",
        "{value:1.2.3}",
    ] {
        assert!(
            top_level_keys(invalid).is_err(),
            "invalid JSON5 accepted: {invalid}"
        );
    }
}

#[test]
fn root_discovery_prefers_nearest_non_extending_turbo_config() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    write(&directory.path().join("turbo.json"), r#"{"tasks":{}}"#)?;
    write(
        &directory.path().join("apps/web/turbo.json"),
        r#"{"extends":["//"]}"#,
    )?;
    write(&directory.path().join("apps/web/src/file.txt"), "fixture")?;

    let root = find_turbo_root(&directory.path().join("apps/web/src"));
    assert_eq!(root.as_deref(), Some(directory.path()));
    Ok(())
}

#[test]
fn comparison_selection_matches_typescript_contract() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let reporter = RecordingReporter::default();
    let no_calls = MockRunner::new(Vec::<CommandOutput>::new());

    let head = get_comparison(
        "test-workspace",
        None,
        &Environment::default(),
        None,
        directory.path(),
        &no_calls,
        &reporter,
        Duration::from_secs(1),
        1_024,
    )
    .ok_or_else(|| std::io::Error::other("missing HEAD comparison"))?;
    assert_eq!(head.reference, "HEAD^");
    assert_eq!(head.kind, ComparisonKind::HeadRelative);

    let custom = get_comparison(
        "test-workspace",
        Some("HEAD^2"),
        &Environment::default(),
        None,
        directory.path(),
        &no_calls,
        &reporter,
        Duration::from_secs(1),
        1_024,
    )
    .ok_or_else(|| std::io::Error::other("missing fallback comparison"))?;
    assert_eq!(custom.reference, "HEAD^2");
    assert_eq!(custom.kind, ComparisonKind::CustomFallback);

    let vercel_without_previous = Environment {
        vercel: true,
        git_commit_ref: Some("my-branch".to_owned()),
        ..Environment::default()
    };
    assert!(
        get_comparison(
            "test-workspace",
            None,
            &vercel_without_previous,
            None,
            directory.path(),
            &no_calls,
            &reporter,
            Duration::from_secs(1),
            1_024,
        )
        .is_none()
    );
    Ok(())
}

#[test]
fn previous_deployment_comparison_validates_git_object_without_option_confusion()
-> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let runner = MockRunner::new([successful_output("")]);
    let reporter = RecordingReporter::default();
    let git = directory.path().join("git");
    let environment = Environment {
        vercel: true,
        previous_sha: Some("mygitsha".to_owned()),
        git_commit_ref: Some("my-branch".to_owned()),
        ..Environment::default()
    };

    let comparison = get_comparison(
        "test-workspace",
        None,
        &environment,
        Some(&git),
        directory.path(),
        &runner,
        &reporter,
        Duration::from_secs(1),
        1_024,
    )
    .ok_or_else(|| std::io::Error::other("missing previous deployment comparison"))?;
    assert_eq!(comparison.reference, "mygitsha");
    assert_eq!(comparison.kind, ComparisonKind::PreviousDeploy);

    let calls = runner.calls()?;
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].args,
        vec![
            OsString::from("cat-file"),
            OsString::from("-e"),
            OsString::from("--end-of-options"),
            OsString::from("mygitsha^{object}"),
        ]
    );
    Ok(())
}

#[test]
fn unreachable_previous_deployment_uses_custom_fallback() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let runner = MockRunner::new([failed_output("not a valid object")]);
    let reporter = RecordingReporter::default();
    let git = directory.path().join("git");
    let environment = Environment {
        vercel: true,
        previous_sha: Some("mygitsha".to_owned()),
        git_commit_ref: Some("my-branch".to_owned()),
        ..Environment::default()
    };

    let comparison = get_comparison(
        "test-workspace",
        Some("HEAD^2"),
        &environment,
        Some(&git),
        directory.path(),
        &runner,
        &reporter,
        Duration::from_secs(1),
        1_024,
    )
    .ok_or_else(|| std::io::Error::other("missing custom fallback"))?;
    assert_eq!(comparison.reference, "HEAD^2");
    assert_eq!(comparison.kind, ComparisonKind::CustomFallback);
    assert!(reporter.contains("is unreachable")?);
    Ok(())
}

#[test]
fn workspace_is_inferred_from_directory_package_json() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    write(
        &directory.path().join("package.json"),
        r#"{"name":"@scope/web"}"#,
    )?;
    let reporter = RecordingReporter::default();
    let workspace = get_workspace(None, directory.path(), &reporter);
    assert_eq!(workspace.as_deref(), Some("@scope/web"));
    assert!(reporter.contains("Inferred workspace")?);
    Ok(())
}

#[test]
fn turbo_version_prefers_argument_then_dependency_then_config_shape() -> Result<(), Box<dyn Error>>
{
    let directory = tempfile::tempdir()?;
    let reporter = RecordingReporter::default();
    write(
        &directory.path().join("package.json"),
        r#"{"dependencies":{"turbo":"^2.4.0"}}"#,
    )?;
    write(
        &directory.path().join("turbo.json"),
        r#"{pipeline:{build:{}}}"#,
    )?;

    assert_eq!(
        infer_turbo_version(Some("2.9.0"), directory.path(), &reporter).as_deref(),
        Some("2.9.0")
    );
    assert_eq!(
        infer_turbo_version(None, directory.path(), &reporter).as_deref(),
        Some("^2.4.0")
    );

    write(
        &directory.path().join("package.json"),
        r#"{"dependencies":{"turbo":"catalog:"}}"#,
    )?;
    assert_eq!(
        infer_turbo_version(None, directory.path(), &reporter).as_deref(),
        Some("^1")
    );
    Ok(())
}

#[test]
fn known_error_messages_match_typescript_categories() {
    let missing = classify_error("reading pnpm-lock.yaml: no such file or directory");
    assert_eq!(missing.level, ErrorLevel::Warn);
    assert_eq!(missing.code, ErrorCode::MissingLockfile);

    let package_manager =
        classify_error("run failed: We did not detect an in-use package manager for your project");
    assert_eq!(package_manager.code, ErrorCode::NoPackageManager);

    let parent = classify_error("failed to resolve packages to run: commit HEAD^ does not exist");
    assert_eq!(parent.code, ErrorCode::UnreachableParent);

    let invalid = classify_error("fatal: unknown revision 'removed-branch'");
    assert_eq!(invalid.code, ErrorCode::InvalidComparison);

    let unknown = classify_error("unexpected failure");
    assert_eq!(unknown.level, ErrorLevel::Error);
    assert_eq!(unknown.code, ErrorCode::UnknownError);
    assert_eq!(unknown.code.as_str(), "UNKNOWN_ERROR");
}

#[test]
fn empty_dry_run_package_list_skips_build() -> Result<(), Box<dyn Error>> {
    let (_directory, options) = fixture()?;
    let runner = MockRunner::new([
        successful_output("2.10.13-canary.1\n"),
        successful_output(r#"{"packages":[]}"#),
    ]);
    let reporter = RecordingReporter::default();
    let environment = Environment {
        vercel: true,
        turbo_force: false,
        commit_message: Some("ordinary commit".to_owned()),
        previous_sha: None,
        git_commit_ref: Some("main".to_owned()),
    };

    let decision = evaluate(&options, &environment, &runner, &reporter);
    assert_eq!(decision, BuildDecision::Skip);
    assert_eq!(decision.exit_code(), 0);
    assert!(reporter.contains("not affected")?);

    let calls = runner.calls()?;
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].args, vec![OsString::from("--version")]);
    assert_eq!(
        calls[1].args,
        vec![
            OsString::from("run"),
            OsString::from("build"),
            OsString::from("--filter=web...[main]"),
            OsString::from("--dry=json"),
        ]
    );
    Ok(())
}

#[test]
fn affected_package_list_proceeds_with_deployment() -> Result<(), Box<dyn Error>> {
    let (_directory, options) = fixture()?;
    let runner = MockRunner::new([
        successful_output("2.10.13-canary.1\n"),
        successful_output(r#"{"packages":["web","shared"]}"#),
    ]);
    let reporter = RecordingReporter::default();
    let environment = Environment {
        vercel: true,
        turbo_force: false,
        commit_message: Some("ordinary commit".to_owned()),
        previous_sha: None,
        git_commit_ref: None,
    };

    let decision = evaluate(&options, &environment, &runner, &reporter);
    assert_eq!(decision, BuildDecision::Deploy);
    assert_eq!(decision.exit_code(), 1);
    assert!(reporter.contains("1 dependency")?);
    Ok(())
}

#[test]
fn malformed_or_null_dry_run_output_fails_open() -> Result<(), Box<dyn Error>> {
    for stdout in ["not json", "null"] {
        let (_directory, options) = fixture()?;
        let runner = MockRunner::new([
            successful_output("2.10.13-canary.1\n"),
            successful_output(stdout),
        ]);
        let reporter = RecordingReporter::default();
        let environment = Environment {
            vercel: true,
            turbo_force: false,
            commit_message: Some("ordinary commit".to_owned()),
            previous_sha: None,
            git_commit_ref: None,
        };
        assert_eq!(
            evaluate(&options, &environment, &runner, &reporter),
            BuildDecision::Deploy
        );
    }
    Ok(())
}

#[test]
fn unspecified_task_uses_build_before_commit_directive() -> Result<(), Box<dyn Error>> {
    let (_directory, mut options) = fixture()?;
    options.task = None;
    options.git_path = Some(Path::new("relative/git").to_path_buf());
    let runner = MockRunner::new(Vec::<CommandOutput>::new());
    let reporter = RecordingReporter::default();
    let environment = Environment {
        vercel: true,
        commit_message: Some("docs: skip [skip ci]".to_owned()),
        ..Environment::default()
    };

    assert_eq!(
        evaluate(&options, &environment, &runner, &reporter),
        BuildDecision::Skip
    );
    assert!(reporter.contains("Using \"build\" as the task as it was unspecified")?);
    Ok(())
}

#[test]
fn force_environment_still_applies_directory_fallback_first() -> Result<(), Box<dyn Error>> {
    let (directory, mut options) = fixture()?;
    options.directory = Some(directory.path().join("does-not-exist"));
    let runner = MockRunner::new(Vec::<CommandOutput>::new());
    let reporter = RecordingReporter::default();
    let environment = Environment {
        turbo_force: true,
        ..Environment::default()
    };

    assert_eq!(
        evaluate(&options, &environment, &runner, &reporter),
        BuildDecision::Deploy
    );
    assert!(reporter.contains("does not exist, using current directory")?);
    assert_eq!(runner.call_count()?, 0);
    Ok(())
}

#[test]
fn force_environment_variable_bypasses_analysis_and_deploys() -> Result<(), Box<dyn Error>> {
    let (_directory, options) = fixture()?;
    let runner = MockRunner::new(Vec::<CommandOutput>::new());
    let reporter = RecordingReporter::default();
    let environment = Environment {
        turbo_force: true,
        ..Environment::default()
    };

    assert_eq!(
        evaluate(&options, &environment, &runner, &reporter),
        BuildDecision::Deploy
    );
    assert_eq!(runner.call_count()?, 0);
    Ok(())
}

#[test]
fn vercel_commit_directive_does_not_require_git_or_turbo() -> Result<(), Box<dyn Error>> {
    let (_directory, mut options) = fixture()?;
    options.git_path = Some(Path::new("relative/git").to_path_buf());
    options.turbo_path = Some(Path::new("relative/turbo").to_path_buf());
    options.fallback = Some("--invalid-option".to_owned());
    let runner = MockRunner::new(Vec::<CommandOutput>::new());
    let reporter = RecordingReporter::default();
    let environment = Environment {
        vercel: true,
        commit_message: Some("docs: skip [skip ci]".to_owned()),
        ..Environment::default()
    };

    assert_eq!(
        evaluate(&options, &environment, &runner, &reporter),
        BuildDecision::Skip
    );
    assert_eq!(runner.call_count()?, 0);
    Ok(())
}

#[test]
fn trusted_turbo_version_mismatch_fails_open_before_analysis() -> Result<(), Box<dyn Error>> {
    let (_directory, options) = fixture()?;
    let runner = MockRunner::new([successful_output("1.13.4\n")]);
    let reporter = RecordingReporter::default();
    let environment = Environment {
        vercel: true,
        commit_message: Some("ordinary commit".to_owned()),
        ..Environment::default()
    };

    assert_eq!(
        evaluate(&options, &environment, &runner, &reporter),
        BuildDecision::Deploy
    );
    assert_eq!(runner.call_count()?, 1);
    assert!(reporter.contains("does not satisfy")?);
    Ok(())
}
