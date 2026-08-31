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
