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
fn turbo_version_prefers_argument_then_dependency_then_config_shape() -> Result<(), Box<dyn Error>> {
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

    let package_manager = classify_error(
        "run failed: We did not detect an in-use package manager for your project",
    );
    assert_eq!(package_manager.code, ErrorCode::NoPackageManager);

    let parent = classify_error(
        "failed to resolve packages to run: commit HEAD^ does not exist",
    );
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

