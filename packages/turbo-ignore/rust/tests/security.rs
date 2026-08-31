use std::{
    error::Error,
    fs,
    path::Path,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use pretty_assertions::assert_eq;
use turbo_ignore::{
    BuildDecision, CommandRunner, CommandSpec, Environment, Options, Reporter, SystemCommandRunner,
    check_commit, evaluate, sanitize_for_log, validate_ref, validate_task,
    validate_version_selector, validate_workspace,
};

#[derive(Debug, Default)]
struct SilentReporter;

impl Reporter for SilentReporter {
    fn info(&self, _message: &str) {}
    fn warn(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
    fn log(&self, _message: &str) {}
}

fn write(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

fn make_executable(path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
    write(path, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = fs::metadata(path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions)?;
    }
    Ok(())
}

#[test]
fn remote_git_tarball_alias_and_file_specs_are_rejected() {
    for value in [
        "https://attacker.invalid/turbo.tgz",
        "git+ssh://attacker.invalid/repo.git",
        "file:../../payload",
        "../payload",
        "npm:evil-package@1.0.0",
        "github:owner/repo",
        "@scope/not-turbo",
        "latest",
    ] {
        assert!(
            validate_version_selector(value).is_err(),
            "unsafe selector should be rejected: {value}"
        );
    }

    for value in ["2.10.13", "2.10.13-canary.1", "^2", "~2.8.0", ">=2.0.0"] {
        assert!(
            validate_version_selector(value).is_ok(),
            "safe selector should be accepted: {value}"
        );
    }
}

#[test]
fn leading_dash_refs_are_rejected_before_git_invocation() {
    assert!(validate_ref("--batch").is_err());
    assert!(validate_ref("-p").is_err());
    assert!(validate_ref("HEAD^").is_ok());
    assert!(validate_ref("main").is_ok());
}

#[test]
fn turbo_filter_and_option_injection_inputs_are_rejected() {
    for workspace in [
        "web]...api",
        "web...[HEAD^]",
        "@scope/name/extra",
        "web,api",
        "web\nspoofed",
    ] {
        assert!(
            validate_workspace(workspace).is_err(),
            "unsafe workspace should be rejected: {workspace:?}"
        );
    }
    for workspace in ["web", "web-app", "@scope/web", "private.package"] {
        assert!(
            validate_workspace(workspace).is_ok(),
            "safe workspace should be accepted: {workspace}"
        );
    }

    for task in ["--filter=api", "build task", "build]", "build\nspoofed"] {
        assert!(
            validate_task(task).is_err(),
            "unsafe task should be rejected: {task:?}"
        );
    }
    for task in ["build", "workspace#build", "//#build", "build:production"] {
        assert!(
            validate_task(task).is_ok(),
            "safe task should be accepted: {task}"
        );
    }

    for reference in ["main]...api", "main..other", "ref:path", "@{upstream}"] {
        assert!(
            validate_ref(reference).is_err(),
            "unsafe ref should be rejected: {reference}"
        );
    }
}

#[test]
fn terminal_control_characters_are_escaped() {
    assert_eq!(
        sanitize_for_log("web\u{1b}[2J\r\nspoofed"),
        "web\\u{1b}[2J\\r\\nspoofed"
    );
}

#[test]
fn multiple_only_directives_fail_open_as_conflict() {
    let decision = check_commit("web", "[vercel only api] [vercel only web]");
    assert_eq!(decision.result, turbo_ignore::CommitResult::Conflict);
}

#[test]
fn conflicting_directives_force_deployment_without_subprocesses() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let root = directory.path();
    write(
        &root.join("package.json"),
        r#"{"name":"root","devDependencies":{"turbo":"2.10.13"}}"#,
    )?;
    write(&root.join("turbo.json"), r#"{"tasks":{}}"#)?;
    write(&root.join("apps/web/package.json"), r#"{"name":"web"}"#)?;

    #[derive(Debug, Default)]
    struct CountingRunner {
        calls: AtomicUsize,
    }
    impl CommandRunner for CountingRunner {
        fn run(
            &self,
            spec: &CommandSpec,
        ) -> Result<turbo_ignore::CommandOutput, turbo_ignore::ProcessError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(turbo_ignore::ProcessError::ReaderChannelClosed {
                program: spec.program.clone(),
            })
        }
    }

    let options = Options {
        workspace: Some("web".to_owned()),
        task: Some("build".to_owned()),
        directory: Some(root.join("apps/web")),
        current_directory: Some(root.to_path_buf()),
        ..Options::default()
    };
    let environment = Environment {
        vercel: true,
        commit_message: Some("[vercel deploy] [skip ci]".to_owned()),
        ..Environment::default()
    };
    let runner = CountingRunner::default();

    assert_eq!(
        evaluate(&options, &environment, &runner, &SilentReporter),
        BuildDecision::Deploy
    );
    assert_eq!(runner.calls.load(Ordering::Relaxed), 0);
    Ok(())
}

#[test]
fn invalid_workspace_cannot_control_workspace_scoped_directives() {
    let decision = check_commit(
        "web] [vercel skip api",
        "release [vercel skip web] [vercel skip api]",
    );
    assert_eq!(decision.result, turbo_ignore::CommitResult::Conflict);
}

#[test]
fn global_skip_directive_does_not_use_invalid_filter_inputs() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let root = directory.path();
    write(
        &root.join("package.json"),
        r#"{"name":"root","devDependencies":{"turbo":"2.10.13"}}"#,
    )?;
    write(&root.join("turbo.json"), r#"{"tasks":{}}"#)?;
    write(&root.join("apps/web/package.json"), r#"{"name":"web"}"#)?;

    #[derive(Debug)]
    struct NoCalls;
    impl CommandRunner for NoCalls {
        fn run(
            &self,
            spec: &CommandSpec,
        ) -> Result<turbo_ignore::CommandOutput, turbo_ignore::ProcessError> {
            Err(turbo_ignore::ProcessError::ReaderChannelClosed {
                program: spec.program.clone(),
            })
        }
    }

    let options = Options {
        workspace: Some("web]...[main".to_owned()),
        task: Some("--filter=api".to_owned()),
        fallback: Some("--invalid".to_owned()),
        directory: Some(root.join("apps/web")),
        git_path: Some(Path::new("relative/git").to_path_buf()),
        turbo_path: Some(Path::new("relative/turbo").to_path_buf()),
        current_directory: Some(root.to_path_buf()),
        ..Options::default()
    };
    let environment = Environment {
        vercel: true,
        commit_message: Some("docs: skip [skip ci]".to_owned()),
        ..Environment::default()
    };

    assert_eq!(
        evaluate(&options, &environment, &NoCalls, &SilentReporter),
        BuildDecision::Skip
    );
    Ok(())
}

#[test]
fn unsafe_dependency_selector_causes_deployment_without_running_turbo() -> Result<(), Box<dyn Error>>
{
    let directory = tempfile::tempdir()?;
    let root = directory.path();
    write(
        &root.join("package.json"),
        r#"{"name":"root","devDependencies":{"turbo":"https://attacker.invalid/payload.tgz"}}"#,
    )?;
    write(&root.join("turbo.json"), r#"{"tasks":{}}"#)?;
    write(&root.join("apps/web/package.json"), r#"{"name":"web"}"#)?;
    let git = root.join("git");
    let turbo = root.join("turbo");
    make_executable(&git, "#!/bin/sh\nexit 0\n")?;
    make_executable(&turbo, "#!/bin/sh\nexit 0\n")?;

    #[derive(Debug, Default)]
    struct CountingRunner {
        calls: AtomicUsize,
    }
    impl CommandRunner for CountingRunner {
        fn run(
            &self,
            spec: &CommandSpec,
        ) -> Result<turbo_ignore::CommandOutput, turbo_ignore::ProcessError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Err(turbo_ignore::ProcessError::ReaderChannelClosed {
                program: spec.program.clone(),
            })
        }
    }

    let options = Options {
        workspace: Some("web".to_owned()),
        task: Some("build".to_owned()),
        fallback: Some("main".to_owned()),
        directory: Some(root.join("apps/web")),
        turbo_path: Some(turbo),
        git_path: Some(git),
        timeout: Duration::from_secs(1),
        current_directory: Some(root.to_path_buf()),
        ..Options::default()
    };
    let environment = Environment {
        vercel: true,
        commit_message: Some("ordinary commit".to_owned()),
        ..Environment::default()
    };

    let runner = CountingRunner::default();
    let decision = evaluate(&options, &environment, &runner, &SilentReporter);
    assert_eq!(decision, BuildDecision::Deploy);
    assert_eq!(runner.calls.load(Ordering::Relaxed), 0);
    Ok(())
}

#[test]
fn configured_directory_must_be_a_directory() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let file = directory.path().join("not-a-directory");
    write(&file, "content")?;

    #[derive(Debug)]
    struct NoCalls;
    impl CommandRunner for NoCalls {
        fn run(
            &self,
            spec: &CommandSpec,
        ) -> Result<turbo_ignore::CommandOutput, turbo_ignore::ProcessError> {
            Err(turbo_ignore::ProcessError::ReaderChannelClosed {
                program: spec.program.clone(),
            })
        }
    }

    let options = Options {
        directory: Some(file),
        current_directory: Some(directory.path().to_path_buf()),
        ..Options::default()
    };
    assert_eq!(
        evaluate(&options, &Environment::default(), &NoCalls, &SilentReporter,),
        BuildDecision::Deploy
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn discovery_rejects_symlinked_configuration_files() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let directory = tempfile::tempdir()?;
    let outside = tempfile::tempdir()?;
    write(&outside.path().join("turbo.json"), r#"{"tasks":{}}"#)?;
    symlink(
        outside.path().join("turbo.json"),
        directory.path().join("turbo.json"),
    )?;
    fs::create_dir_all(directory.path().join("apps/web"))?;

    assert!(turbo_ignore::find_turbo_root(&directory.path().join("apps/web")).is_none());
    Ok(())
}

#[cfg(unix)]
#[test]
fn subprocess_timeout_terminates_hung_child() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let script = directory.path().join("hang.sh");
    make_executable(&script, "#!/bin/sh\nwhile :; do :; done\n")?;
    let spec = CommandSpec {
        program: script,
        args: Vec::new(),
        cwd: directory.path().to_path_buf(),
        timeout: Duration::from_millis(50),
        max_output_bytes: 1_024,
    };

    let result = SystemCommandRunner.run(&spec);
    assert!(matches!(
        result,
        Err(turbo_ignore::ProcessError::Timeout { .. })
    ));
    Ok(())
}

#[cfg(unix)]
#[test]
fn subprocess_output_limit_terminates_noisy_child() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let script = directory.path().join("noisy.sh");
    make_executable(
        &script,
        "#!/bin/sh\nwhile true; do printf '0123456789abcdef'; done\n",
    )?;
    let spec = CommandSpec {
        program: script,
        args: Vec::new(),
        cwd: directory.path().to_path_buf(),
        timeout: Duration::from_secs(2),
        max_output_bytes: 128,
    };

    let result = SystemCommandRunner.run(&spec);
    assert!(matches!(
        result,
        Err(turbo_ignore::ProcessError::OutputTooLarge { .. })
    ));
    Ok(())
}
