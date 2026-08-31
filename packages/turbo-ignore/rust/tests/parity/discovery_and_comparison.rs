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
fn previous_deployment_comparison_validates_git_object_without_option_confusion(
) -> Result<(), Box<dyn Error>> {
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

