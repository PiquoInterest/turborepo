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
fn unsafe_dependency_selector_causes_deployment_without_running_turbo() -> Result<(), Box<dyn Error>> {
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

