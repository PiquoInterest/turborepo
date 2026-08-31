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
        evaluate(
            &options,
            &Environment::default(),
            &NoCalls,
            &SilentReporter,
        ),
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

    assert!(
        turbo_ignore::find_turbo_root(&directory.path().join("apps/web")).is_none()
    );
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
