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

